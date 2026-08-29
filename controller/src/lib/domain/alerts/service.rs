use crate::domain::alerts::models::{
    AlertHistory, AlertIncident, AlertRule, AlertRuleError, CreateAlertRuleRequest,
    MAX_HISTORY_EVENTS, MAX_RULES_PER_USER, NewAlertEvent,
};
use crate::domain::alerts::port::{AlertsRepository, AlertsService};
use chrono::{DateTime, Utc};
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
        now: DateTime<Utc>,
    ) -> Vec<AlertIncident> {
        // The state machine works in whole unix seconds; the history keeps the
        // full-precision instant.
        let now_secs = now.timestamp();
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
                let event = state.observe(&cond, value, now_secs);
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

        // Record what we are about to notify about, so the dashboard can show
        // the same transitions as a history. A failed write must not cost the
        // user the notification, so it is logged rather than propagated.
        if !incidents.is_empty() {
            let records: Vec<NewAlertEvent> = incidents
                .iter()
                .map(|i| NewAlertEvent::from_incident(i, hostname, project, now))
                .collect();
            if let Err(e) = self.repository.record_events(user_id, &records).await {
                error!("alert evaluation: recording history failed: {e:?}");
            }
        }
        incidents
    }

    async fn history(
        &self,
        user_id: &str,
        session_id: &str,
        now: DateTime<Utc>,
    ) -> Result<AlertHistory, AlertRuleError> {
        let new_since = self.repository.mark_seen(user_id, session_id, now).await?;
        let events = self
            .repository
            .list_events(user_id, MAX_HISTORY_EVENTS)
            .await?;
        Ok(AlertHistory { events, new_since })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::alerts::models::AlertEventRecord;
    use hoister_shared::alerts::{AlertEvent, AlertEventKind, AlertMetric, AlertState};
    use std::sync::{Arc, Mutex};

    /// The persisted "last login" watermark: current session, when it first
    /// looked, and when its predecessor first looked.
    type SeenRow = (String, DateTime<Utc>, Option<DateTime<Utc>>);

    /// In-memory repository double so evaluate() can be exercised without a DB.
    #[derive(Clone, Default)]
    struct MemRepo {
        rules: Arc<Mutex<Vec<AlertRule>>>,
        states: Arc<Mutex<HashMap<(uuid::Uuid, ServiceName), AlertState>>>,
        events: Arc<Mutex<Vec<(String, AlertEventRecord)>>>,
        seen: Arc<Mutex<HashMap<String, SeenRow>>>,
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

        async fn record_events(
            &self,
            user_id: &str,
            events: &[NewAlertEvent],
        ) -> Result<(), AlertRuleError> {
            let mut stored = self.events.lock().unwrap();
            for e in events {
                stored.push((
                    user_id.to_string(),
                    AlertEventRecord {
                        id: uuid::Uuid::new_v4(),
                        rule_id: Some(e.rule_id),
                        kind: e.kind,
                        metric: e.metric,
                        threshold: e.threshold,
                        for_seconds: e.for_seconds,
                        value: e.value,
                        hostname: e.hostname.clone(),
                        project: e.project.clone(),
                        service: e.service.clone(),
                        triggered_at: e.triggered_at,
                    },
                ));
            }
            Ok(())
        }

        async fn list_events(
            &self,
            user_id: &str,
            limit: i64,
        ) -> Result<Vec<AlertEventRecord>, AlertRuleError> {
            let stored = self.events.lock().unwrap();
            let mut out: Vec<AlertEventRecord> = stored
                .iter()
                .filter(|(u, _)| u == user_id)
                .map(|(_, e)| e.clone())
                .collect();
            out.sort_by(|a, b| b.triggered_at.cmp(&a.triggered_at));
            out.truncate(limit.max(0) as usize);
            Ok(out)
        }

        async fn mark_seen(
            &self,
            user_id: &str,
            session_id: &str,
            now: DateTime<Utc>,
        ) -> Result<Option<DateTime<Utc>>, AlertRuleError> {
            let mut seen = self.seen.lock().unwrap();
            let row = match seen.get(user_id) {
                None => (session_id.to_string(), now, None),
                Some((sid, started, previous)) if sid == session_id => {
                    (sid.clone(), *started, *previous)
                }
                Some((_, started, _)) => (session_id.to_string(), now, Some(*started)),
            };
            let previous = row.2;
            seen.insert(user_id.to_string(), row);
            Ok(previous)
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

    /// Unix seconds as an instant, for the injected evaluation clock.
    fn at(secs: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(secs, 0).expect("valid timestamp")
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
            .evaluate("u1", &host, &project, &samples(&[("web", 95.0)]), at(0))
            .await;
        assert!(first.is_empty(), "first breach only pends");

        // A second evaluation past the window fires — the pending state was
        // persisted through the repository between calls.
        let second = service
            .evaluate("u1", &host, &project, &samples(&[("web", 95.0)]), at(120))
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
                .evaluate("u1", &host, &project, &s, at(0))
                .await
                .is_empty()
        );
        assert!(
            service
                .evaluate("u1", &host, &project, &s, at(999))
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
        service.evaluate("u2", &host, &project, &s, at(0)).await;
        assert!(
            service
                .evaluate("u2", &host, &project, &s, at(120))
                .await
                .is_empty(),
            "u1's rules must not fire for u2's metrics"
        );
    }

    /// Firing and resolving both land in the history, in newest-first order,
    /// scoped to the user whose metrics produced them.
    #[tokio::test]
    async fn evaluate_records_every_transition_it_notifies_about() {
        let service = Service::new(MemRepo::default());
        service
            .create_rule("u1", create_req(80.0, None))
            .await
            .unwrap();
        let host = HostName::new("h1");
        let project = ProjectName("p1".into());

        service
            .evaluate("u1", &host, &project, &samples(&[("web", 95.0)]), at(0))
            .await;
        service
            .evaluate("u1", &host, &project, &samples(&[("web", 95.0)]), at(120))
            .await;
        // Two samples below the threshold, `for_seconds` apart, resolve it.
        service
            .evaluate("u1", &host, &project, &samples(&[("web", 10.0)]), at(180))
            .await;
        service
            .evaluate("u1", &host, &project, &samples(&[("web", 10.0)]), at(300))
            .await;

        let history = service
            .history("u1", "session-a", Utc::now())
            .await
            .unwrap();
        let kinds: Vec<AlertEventKind> = history.events.iter().map(|e| e.kind).collect();
        assert_eq!(kinds, vec![AlertEventKind::Resolved, AlertEventKind::Fired]);
        let fired = &history.events[1];
        assert_eq!(fired.service.as_str(), "web");
        assert_eq!(fired.hostname.as_str(), "h1");
        assert_eq!(fired.project.as_str(), "p1");
        assert_eq!(fired.value, 95.0);
        assert_eq!(fired.triggered_at.timestamp(), 120);

        assert!(
            service
                .history("u2", "session-a", Utc::now())
                .await
                .unwrap()
                .events
                .is_empty(),
            "history is per user"
        );
    }

    /// The "new since your last login" cutoff only moves when the session
    /// changes, so revisiting the page within one login keeps highlighting the
    /// same entries.
    #[tokio::test]
    async fn history_highlights_what_fired_since_the_previous_login() {
        let service = Service::new(MemRepo::default());
        service
            .create_rule("u1", create_req(80.0, None))
            .await
            .unwrap();
        let host = HostName::new("h1");
        let project = ProjectName("p1".into());
        let t0 = DateTime::from_timestamp(1_000_000, 0).unwrap();

        // First login: nothing to compare against, so nothing is highlighted.
        let first = service.history("u1", "session-a", t0).await.unwrap();
        assert_eq!(first.new_since, None);
        assert_eq!(first.new_count(), 0);

        // An alert fires while that first session is still the last one seen.
        let fired_at = t0.timestamp() + 600;
        service
            .evaluate(
                "u1",
                &host,
                &project,
                &samples(&[("web", 95.0)]),
                at(fired_at),
            )
            .await;
        service
            .evaluate(
                "u1",
                &host,
                &project,
                &samples(&[("web", 95.0)]),
                at(fired_at + 120),
            )
            .await;

        // Same session: still nothing "new since the last login".
        let same = service
            .history("u1", "session-a", t0 + chrono::Duration::seconds(900))
            .await
            .unwrap();
        assert_eq!(same.events.len(), 1);
        assert_eq!(same.new_count(), 0);

        // Next login: the alert fired while they were away, so it is new — and
        // it stays new for the rest of that session.
        let later = t0 + chrono::Duration::hours(5);
        let next = service.history("u1", "session-b", later).await.unwrap();
        assert_eq!(next.new_since, Some(t0));
        assert_eq!(next.new_count(), 1);
        assert!(next.is_new(&next.events[0]));

        let revisit = service
            .history("u1", "session-b", later + chrono::Duration::minutes(5))
            .await
            .unwrap();
        assert_eq!(revisit.new_since, Some(t0));
        assert_eq!(revisit.new_count(), 1, "the cutoff holds within a login");

        // A third login: the alert now predates the previous session, so it
        // has been seen and is no longer highlighted.
        let third = service
            .history("u1", "session-c", later + chrono::Duration::hours(5))
            .await
            .unwrap();
        assert_eq!(third.new_since, Some(later));
        assert_eq!(third.new_count(), 0);
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
