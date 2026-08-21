//! Agent-side evaluation of `[[alert]]` rules against the per-minute stats
//! samples. Rules and the firing state machine live in
//! [`hoister_shared::alerts`] (shared with the cloud controller); this module
//! holds the per-(rule, service) states across ticks and turns transitions
//! into chatterbox messages. Delivery stays with the caller so the engine is
//! testable without network transports.

use crate::config::AlertRule;
use chatterbox::message::Message;
use hoister_shared::alerts::{AlertCondition, AlertState, AlertTarget, alert_message};
use hoister_shared::wire::ContainerMetricSample;
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;

struct Rule {
    cond: AlertCondition,
    /// Only evaluate for this service; `None` watches every service.
    service: Option<ServiceName>,
}

pub(crate) struct AlertEngine {
    rules: Vec<Rule>,
    /// Firing state per (rule index, service). In-memory only: an agent
    /// restart starts clean, which at worst delays a fire by one `for` window.
    states: HashMap<(usize, ServiceName), AlertState>,
    hostname: HostName,
    project: ProjectName,
}

impl AlertEngine {
    pub(crate) fn new(rules: &[AlertRule], project: ProjectName, hostname: HostName) -> Self {
        Self {
            rules: rules
                .iter()
                .map(|r| Rule {
                    cond: r.condition(),
                    service: r.service.clone(),
                })
                .collect(),
            states: HashMap::new(),
            hostname,
            project,
        }
    }

    /// Feed one tick's samples through every rule; returns the notifications
    /// to dispatch. `now` is unix seconds.
    pub(crate) fn evaluate(
        &mut self,
        samples: &HashMap<ServiceName, ContainerMetricSample>,
        now: i64,
    ) -> Vec<Message> {
        let mut messages = Vec::new();
        for (idx, rule) in self.rules.iter().enumerate() {
            for (service, sample) in samples {
                if rule.service.as_ref().is_some_and(|s| s != service) {
                    continue;
                }
                // No derivable value (e.g. mem_pct without a memory limit):
                // leave the state untouched rather than treating it as 0.
                let Some(value) = rule.cond.metric.value(sample) else {
                    continue;
                };
                let state = self.states.entry((idx, service.clone())).or_default();
                if let Some(event) = state.observe(&rule.cond, value, now) {
                    let target = AlertTarget {
                        hostname: self.hostname.as_str(),
                        project: self.project.as_str(),
                        service: service.as_str(),
                    };
                    messages.push(alert_message(&rule.cond, &target, &event, None));
                }
            }
        }
        // Forget states of services that stopped reporting (stopped container,
        // removed service) so the map can't grow across redeploys. A firing
        // alert for a vanished service resets silently — its container being
        // gone is surfaced by the state monitor, not by a metrics alert.
        self.states.retain(|(_, svc), _| samples.contains_key(svc));
        messages
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hoister_shared::alerts::AlertMetric;

    fn rule(metric: AlertMetric, threshold: f64, service: Option<&str>) -> AlertRule {
        AlertRule {
            metric,
            threshold,
            for_seconds: 120,
            cooldown_seconds: 0,
            service: service.map(ServiceName::new),
        }
    }

    fn engine(rules: &[AlertRule]) -> AlertEngine {
        AlertEngine::new(rules, ProjectName::new("proj"), HostName::new("host1"))
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

    #[test]
    fn fires_after_sustained_breach_and_resolves() {
        let mut e = engine(&[rule(AlertMetric::CpuPct, 80.0, None)]);
        assert!(e.evaluate(&samples(&[("web", 95.0)]), 0).is_empty());
        let fired = e.evaluate(&samples(&[("web", 95.0)]), 120);
        assert_eq!(fired.len(), 1);
        assert!(
            fired[0].title.starts_with("Alert: cpu"),
            "{}",
            fired[0].title
        );

        assert!(e.evaluate(&samples(&[("web", 10.0)]), 180).is_empty());
        let resolved = e.evaluate(&samples(&[("web", 10.0)]), 300);
        assert_eq!(resolved.len(), 1);
        assert!(
            resolved[0].title.starts_with("Resolved:"),
            "{}",
            resolved[0].title
        );
    }

    #[test]
    fn service_filter_only_watches_that_service() {
        let mut e = engine(&[rule(AlertMetric::CpuPct, 80.0, Some("web"))]);
        let s = samples(&[("web", 95.0), ("db", 95.0)]);
        e.evaluate(&s, 0);
        let fired = e.evaluate(&s, 120);
        assert_eq!(fired.len(), 1);
        assert!(fired[0].body.contains("service web"), "{}", fired[0].body);
    }

    #[test]
    fn unfiltered_rule_tracks_services_independently() {
        let mut e = engine(&[rule(AlertMetric::CpuPct, 80.0, None)]);
        e.evaluate(&samples(&[("web", 95.0), ("db", 10.0)]), 0);
        let fired = e.evaluate(&samples(&[("web", 95.0), ("db", 95.0)]), 120);
        assert_eq!(fired.len(), 1, "db only started breaching at t=120");
    }

    #[test]
    fn vanished_service_state_is_pruned() {
        let mut e = engine(&[rule(AlertMetric::CpuPct, 80.0, None)]);
        e.evaluate(&samples(&[("web", 95.0)]), 0);
        // Service gone for a tick: pending state must not survive.
        e.evaluate(&samples(&[]), 60);
        let fired = e.evaluate(&samples(&[("web", 95.0)]), 120);
        assert!(fired.is_empty(), "window must restart after the gap");
    }
}
