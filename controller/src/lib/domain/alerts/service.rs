use crate::domain::alerts::models::{
    AlertIncident, AlertRule, AlertRuleError, CreateAlertRuleRequest, MAX_RULES_PER_USER,
};
use crate::domain::alerts::port::{AlertsRepository, AlertsService};
use hoister_shared::wire::ContainerMetricSample;
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::error;
use std::collections::HashMap;

#[derive(Clone)]
pub struct Service<AR: AlertsRepository> {
    repository: AR,
}

impl<AR: AlertsRepository> Service<AR> {
    pub fn new(repository: AR) -> Self {
        Self { repository }
    }
}

impl<AR: AlertsRepository> AlertsService for Service<AR> {
    async fn list_rules(&self, user_id: &str) -> Result<Vec<AlertRule>, AlertRuleError> {
        self.repository.list_rules(user_id).await
    }

    async fn create_rule(
        &self,
        user_id: &str,
        req: CreateAlertRuleRequest,
    ) -> Result<AlertRule, AlertRuleError> {
        req.validate()?;
        let existing = self.repository.list_rules(user_id).await?;
        if existing.len() >= MAX_RULES_PER_USER {
            return Err(AlertRuleError::InvalidRule(format!(
                "at most {MAX_RULES_PER_USER} alert rules per account"
            )));
        }
        self.repository.create_rule(user_id, req).await
    }

    async fn delete_rule(
        &self,
        user_id: &str,
        rule_id: uuid::Uuid,
    ) -> Result<bool, AlertRuleError> {
        self.repository.delete_rule(user_id, rule_id).await
    }

    async fn set_enabled(
        &self,
        user_id: &str,
        rule_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<bool, AlertRuleError> {
        self.repository.set_enabled(user_id, rule_id, enabled).await
    }

    async fn evaluate(
        &self,
        user_id: &str,
        hostname: &HostName,
        project: &ProjectName,
        samples: &HashMap<ServiceName, ContainerMetricSample>,
        now: i64,
    ) -> Vec<AlertIncident> {
        let rules = match self.repository.list_rules(user_id).await {
            Ok(rules) => rules,
            Err(e) => {
                error!("alert evaluation: listing rules for {user_id} failed: {e:?}");
                return Vec::new();
            }
        };
        let rules: Vec<AlertRule> = rules
            .into_iter()
            .filter(|r| r.enabled && r.applies_to(hostname, project))
            .collect();
        if rules.is_empty() {
            return Vec::new();
        }

        let rule_ids: Vec<uuid::Uuid> = rules.iter().map(|r| r.id).collect();
        let states = match self
            .repository
            .get_states(&rule_ids, hostname, project)
            .await
        {
            Ok(states) => states,
            Err(e) => {
                // Without the persisted states an evaluation would restart
                // every window and re-fire everything — skip this batch.
                error!("alert evaluation: loading states for {user_id} failed: {e:?}");
                return Vec::new();
            }
        };

        let mut incidents = Vec::new();
        let mut dirty = Vec::new();
        for rule in &rules {
            let cond = rule.condition();
            for (service, sample) in samples {
                if !rule.applies_to_service(service) {
                    continue;
                }
                // No derivable value (e.g. mem_pct without a memory limit):
                // leave the state untouched rather than treating it as 0.
                let Some(value) = cond.metric.value(sample) else {
                    continue;
                };
                let before = states
                    .get(&(rule.id, service.clone()))
                    .copied()
                    .unwrap_or_default();
                let mut state = before;
                let event = state.observe(&cond, value, now);
                if state != before {
                    dirty.push((rule.id, service.clone(), state));
                }
                if let Some(event) = event {
                    incidents.push(AlertIncident {
                        rule: rule.clone(),
                        service: service.clone(),
                        event,
                    });
                }
            }
        }

        if !dirty.is_empty()
            && let Err(e) = self.repository.put_states(hostname, project, &dirty).await
        {
            error!("alert evaluation: persisting states for {user_id} failed: {e:?}");
        }
        incidents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoister_shared::alerts::{AlertEvent, AlertMetric, AlertState};
    use std::sync::{Arc, Mutex};

    /// In-memory repository double so evaluate() can be exercised without a DB.
    #[derive(Clone, Default)]
    struct MemRepo {
        rules: Arc<Mutex<Vec<AlertRule>>>,
        states: Arc<Mutex<HashMap<(uuid::Uuid, ServiceName), AlertState>>>,
    }

    impl AlertsRepository for MemRepo {
        async fn list_rules(&self, user_id: &str) -> Result<Vec<AlertRule>, AlertRuleError> {
            Ok(self
                .rules
                .lock()
                .unwrap()
                .iter()
                .filter(|r| r.user_id == user_id)
                .cloned()
                .collect())
        }

        async fn create_rule(
            &self,
            user_id: &str,
            req: CreateAlertRuleRequest,
        ) -> Result<AlertRule, AlertRuleError> {
            let rule = AlertRule {
                id: uuid::Uuid::new_v4(),
                user_id: user_id.to_string(),
                metric: req.metric,
                threshold: req.threshold,
                for_seconds: req.for_seconds,
                cooldown_seconds: req.cooldown_seconds,
                hostname: req.hostname,
                project: req.project,
                service: req.service,
                enabled: true,
                created_at: "now".into(),
            };
            self.rules.lock().unwrap().push(rule.clone());
            Ok(rule)
        }

        async fn delete_rule(
            &self,
            user_id: &str,
            rule_id: uuid::Uuid,
        ) -> Result<bool, AlertRuleError> {
            let mut rules = self.rules.lock().unwrap();
            let before = rules.len();
            rules.retain(|r| !(r.id == rule_id && r.user_id == user_id));
            Ok(rules.len() < before)
        }

        async fn set_enabled(
            &self,
            user_id: &str,
            rule_id: uuid::Uuid,
            enabled: bool,
        ) -> Result<bool, AlertRuleError> {
            let mut rules = self.rules.lock().unwrap();
            for r in rules.iter_mut() {
                if r.id == rule_id && r.user_id == user_id {
                    r.enabled = enabled;
                    return Ok(true);
                }
            }
            Ok(false)
        }

        async fn get_states(
            &self,
            rule_ids: &[uuid::Uuid],
            _hostname: &HostName,
            _project: &ProjectName,
        ) -> Result<HashMap<(uuid::Uuid, ServiceName), AlertState>, AlertRuleError> {
            Ok(self
                .states
                .lock()
                .unwrap()
                .iter()
                .filter(|((id, _), _)| rule_ids.contains(id))
                .map(|(k, v)| (k.clone(), *v))
                .collect())
        }

        async fn put_states(
            &self,
            _hostname: &HostName,
            _project: &ProjectName,
            states: &[(uuid::Uuid, ServiceName, AlertState)],
        ) -> Result<(), AlertRuleError> {
            let mut map = self.states.lock().unwrap();
            for (id, service, state) in states {
                map.insert((*id, service.clone()), *state);
            }
            Ok(())
        }
    }

    fn cpu_sample(cpu: f64) -> ContainerMetricSample {
        ContainerMetricSample {
            cpu_pct: cpu,
            mem_bytes: 0,
            mem_limit_bytes: 0,
            net_rx_bytes: 0,
            net_tx_bytes: 0,
            disk_read_bytes: 0,
            disk_write_bytes: 0,
        }
    }

    fn samples(entries: &[(&str, f64)]) -> HashMap<ServiceName, ContainerMetricSample> {
        entries
            .iter()
            .map(|(name, cpu)| (ServiceName::new(*name), cpu_sample(*cpu)))
            .collect()
    }

    fn create_req(threshold: f64, hostname: Option<&str>) -> CreateAlertRuleRequest {
        CreateAlertRuleRequest {
            metric: AlertMetric::CpuPct,
            threshold,
            for_seconds: 120,
            cooldown_seconds: 0,
            hostname: hostname.map(HostName::new),
            project: None,
            service: None,
        }
    }

    #[tokio::test]
    async fn evaluate_fires_and_state_survives_between_calls() {
        let service = Service::new(MemRepo::default());
        service
            .create_rule("u1", create_req(80.0, None))
            .await
            .unwrap();
        let host = HostName::new("h1");
        let project = ProjectName("p1".into());

        let first = service
            .evaluate("u1", &host, &project, &samples(&[("web", 95.0)]), 0)
            .await;
        assert!(first.is_empty(), "first breach only pends");

        // A second evaluation past the window fires — the pending state was
        // persisted through the repository between calls.
        let second = service
            .evaluate("u1", &host, &project, &samples(&[("web", 95.0)]), 120)
            .await;
        assert_eq!(second.len(), 1);
        assert!(matches!(second[0].event, AlertEvent::Fired { .. }));
        assert_eq!(second[0].service.as_str(), "web");
    }

    #[tokio::test]
    async fn evaluate_skips_rules_scoped_to_other_hosts_and_disabled_rules() {
        let service = Service::new(MemRepo::default());
        let scoped = service
            .create_rule("u1", create_req(80.0, Some("other-host")))
            .await
            .unwrap();
        let disabled = service
            .create_rule("u1", create_req(80.0, None))
            .await
            .unwrap();
        service.set_enabled("u1", disabled.id, false).await.unwrap();
        let _ = scoped;

        let host = HostName::new("h1");
        let project = ProjectName("p1".into());
        let s = samples(&[("web", 95.0)]);
        assert!(
            service
                .evaluate("u1", &host, &project, &s, 0)
                .await
                .is_empty()
        );
        assert!(
            service
                .evaluate("u1", &host, &project, &s, 999)
                .await
                .is_empty(),
            "neither the foreign-host rule nor the disabled rule may fire"
        );
    }

    #[tokio::test]
    async fn evaluate_is_per_user() {
        let service = Service::new(MemRepo::default());
        service
            .create_rule("u1", create_req(80.0, None))
            .await
            .unwrap();
        let host = HostName::new("h1");
        let project = ProjectName("p1".into());
        let s = samples(&[("web", 95.0)]);
        service.evaluate("u2", &host, &project, &s, 0).await;
        assert!(
            service
                .evaluate("u2", &host, &project, &s, 120)
                .await
                .is_empty(),
            "u1's rules must not fire for u2's metrics"
        );
    }

    #[tokio::test]
    async fn create_rule_enforces_validation_and_cap() {
        let service = Service::new(MemRepo::default());
        let mut bad = create_req(80.0, None);
        bad.threshold = f64::INFINITY;
        assert!(service.create_rule("u1", bad).await.is_err());

        for _ in 0..MAX_RULES_PER_USER {
            service
                .create_rule("u1", create_req(80.0, None))
                .await
                .unwrap();
        }
        assert!(
            service
                .create_rule("u1", create_req(80.0, None))
                .await
                .is_err(),
            "rule cap must be enforced"
        );
    }
}
