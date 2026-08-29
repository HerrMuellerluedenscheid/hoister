use chrono::{DateTime, Utc};
use hoister_shared::alerts::{AlertCondition, AlertEvent, AlertEventKind, AlertMetric};
use hoister_shared::{HostName, ProjectName, ServiceName};
use thiserror::Error;

/// Longest accepted `for` window. Beyond a day the rule stops being an alert
/// and becomes a report; it also bounds how long stale pending state matters.
pub const MAX_FOR_SECONDS: u64 = 24 * 60 * 60;
/// Longest accepted re-notification cooldown.
pub const MAX_COOLDOWN_SECONDS: u64 = 30 * 24 * 60 * 60;
/// Rules a single user may have. Evaluation is O(rules × services) per
/// ingested batch, so keep it bounded.
pub const MAX_RULES_PER_USER: usize = 50;
/// How many past alert transitions are kept per user. Transitions are rare by
/// construction (each one is a notification), so this covers a long history
/// while keeping the table from growing without bound.
pub const MAX_HISTORY_EVENTS: i64 = 500;

/// A persisted metric alert rule. The optional scope columns narrow which
/// containers the rule watches; `None` means "any".
#[derive(Debug, Clone)]
pub struct AlertRule {
    pub id: uuid::Uuid,
    pub user_id: String,
    pub metric: AlertMetric,
    pub threshold: f64,
    pub for_seconds: u64,
    pub cooldown_seconds: u64,
    pub hostname: Option<HostName>,
    pub project: Option<ProjectName>,
    pub service: Option<ServiceName>,
    pub enabled: bool,
    pub created_at: String,
}

impl AlertRule {
    pub fn condition(&self) -> AlertCondition {
        AlertCondition {
            metric: self.metric,
            threshold: self.threshold,
            for_seconds: self.for_seconds,
            cooldown_seconds: self.cooldown_seconds,
        }
    }

    /// Whether this rule watches the (host, project) a metrics batch is for.
    pub fn applies_to(&self, hostname: &HostName, project: &ProjectName) -> bool {
        self.hostname.as_ref().is_none_or(|h| h == hostname)
            && self.project.as_ref().is_none_or(|p| p.0 == project.0)
    }

    pub fn applies_to_service(&self, service: &ServiceName) -> bool {
        self.service.as_ref().is_none_or(|s| s == service)
    }
}

/// What a user submits to create a rule. Validated by the service
/// (`threshold` finite, durations within bounds) before it reaches a repo.
#[derive(Debug, Clone)]
pub struct CreateAlertRuleRequest {
    pub metric: AlertMetric,
    pub threshold: f64,
    pub for_seconds: u64,
    pub cooldown_seconds: u64,
    pub hostname: Option<HostName>,
    pub project: Option<ProjectName>,
    pub service: Option<ServiceName>,
}

impl CreateAlertRuleRequest {
    pub fn validate(&self) -> Result<(), AlertRuleError> {
        if !self.threshold.is_finite() || self.threshold < 0.0 {
            return Err(AlertRuleError::InvalidRule(
                "threshold must be a non-negative number".into(),
            ));
        }
        if self.for_seconds > MAX_FOR_SECONDS {
            return Err(AlertRuleError::InvalidRule(format!(
                "`for` must be at most {MAX_FOR_SECONDS} seconds (24h)"
            )));
        }
        if self.cooldown_seconds > MAX_COOLDOWN_SECONDS {
            return Err(AlertRuleError::InvalidRule(format!(
                "cooldown must be at most {MAX_COOLDOWN_SECONDS} seconds (30d)"
            )));
        }
        Ok(())
    }
}

/// One notification-worthy transition produced by evaluating a batch:
/// which rule, on which service, and what happened.
#[derive(Debug, Clone)]
pub struct AlertIncident {
    pub rule: AlertRule,
    pub service: ServiceName,
    pub event: AlertEvent,
}

/// An [`AlertIncident`] on its way into the history table. Carries the rule's
/// condition and target by value rather than by reference to the rule row:
/// the entry has to survive the rule being edited or deleted.
#[derive(Debug, Clone)]
pub struct NewAlertEvent {
    pub rule_id: uuid::Uuid,
    pub kind: AlertEventKind,
    pub metric: AlertMetric,
    pub threshold: f64,
    pub for_seconds: u64,
    pub value: f64,
    pub hostname: HostName,
    pub project: ProjectName,
    pub service: ServiceName,
    pub triggered_at: DateTime<Utc>,
}

impl NewAlertEvent {
    pub fn from_incident(
        incident: &AlertIncident,
        hostname: &HostName,
        project: &ProjectName,
        triggered_at: DateTime<Utc>,
    ) -> Self {
        Self {
            rule_id: incident.rule.id,
            kind: incident.event.kind(),
            metric: incident.rule.metric,
            threshold: incident.rule.threshold,
            for_seconds: incident.rule.for_seconds,
            value: incident.event.value(),
            hostname: hostname.clone(),
            project: project.clone(),
            service: incident.service.clone(),
            triggered_at,
        }
    }
}

/// One recorded alert transition, as listed in the dashboard's alert history.
#[derive(Debug, Clone)]
pub struct AlertEventRecord {
    pub id: uuid::Uuid,
    /// The rule that produced it, or `None` once that rule was deleted.
    pub rule_id: Option<uuid::Uuid>,
    pub kind: AlertEventKind,
    pub metric: AlertMetric,
    pub threshold: f64,
    pub for_seconds: u64,
    pub value: f64,
    pub hostname: HostName,
    pub project: ProjectName,
    pub service: ServiceName,
    pub triggered_at: DateTime<Utc>,
}

/// A user's alert history plus the cutoff the dashboard highlights against.
pub struct AlertHistory {
    /// Newest first.
    pub events: Vec<AlertEventRecord>,
    /// Start of the user's previous login session: everything after it fired
    /// while they were away. `None` on a first-ever login, where there is no
    /// "since" to compare to and nothing is highlighted.
    pub new_since: Option<DateTime<Utc>>,
}

impl AlertHistory {
    /// Whether `event` fired since the user's previous login.
    pub fn is_new(&self, event: &AlertEventRecord) -> bool {
        self.new_since
            .is_some_and(|since| event.triggered_at > since)
    }

    pub fn new_count(&self) -> usize {
        self.events.iter().filter(|e| self.is_new(e)).count()
    }
}

#[derive(Debug, Error)]
pub enum AlertRuleError {
    #[error("Alert rule not found")]
    NotFound,
    #[error("Invalid alert rule: {0}")]
    InvalidRule(String),
    #[error("Unknown error")]
    UnknownError,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(hostname: Option<&str>, project: Option<&str>, service: Option<&str>) -> AlertRule {
        AlertRule {
            id: uuid::Uuid::nil(),
            user_id: "u".into(),
            metric: AlertMetric::CpuPct,
            threshold: 80.0,
            for_seconds: 300,
            cooldown_seconds: 0,
            hostname: hostname.map(HostName::new),
            project: project.map(|p| ProjectName(p.to_string())),
            service: service.map(ServiceName::new),
            enabled: true,
            created_at: "now".into(),
        }
    }

    #[test]
    fn unscoped_rule_applies_everywhere() {
        let r = rule(None, None, None);
        assert!(r.applies_to(&HostName::new("h1"), &ProjectName("p1".into())));
        assert!(r.applies_to_service(&ServiceName::new("web")));
    }

    #[test]
    fn scoped_rule_only_matches_its_target() {
        let r = rule(Some("h1"), Some("p1"), Some("web"));
        assert!(r.applies_to(&HostName::new("h1"), &ProjectName("p1".into())));
        assert!(!r.applies_to(&HostName::new("h2"), &ProjectName("p1".into())));
        assert!(!r.applies_to(&HostName::new("h1"), &ProjectName("p2".into())));
        assert!(!r.applies_to_service(&ServiceName::new("db")));
    }

    #[test]
    fn validation_rejects_nonsense() {
        let mut req = CreateAlertRuleRequest {
            metric: AlertMetric::CpuPct,
            threshold: 80.0,
            for_seconds: 300,
            cooldown_seconds: 0,
            hostname: None,
            project: None,
            service: None,
        };
        assert!(req.validate().is_ok());
        req.threshold = f64::NAN;
        assert!(req.validate().is_err());
        req.threshold = -1.0;
        assert!(req.validate().is_err());
        req.threshold = 80.0;
        req.for_seconds = MAX_FOR_SECONDS + 1;
        assert!(req.validate().is_err());
        req.for_seconds = 0;
        req.cooldown_seconds = MAX_COOLDOWN_SECONDS + 1;
        assert!(req.validate().is_err());
    }
}
