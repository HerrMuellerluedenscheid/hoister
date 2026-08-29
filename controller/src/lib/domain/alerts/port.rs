use crate::domain::alerts::models::{
    AlertEventRecord, AlertHistory, AlertIncident, AlertRule, AlertRuleError,
    CreateAlertRuleRequest, NewAlertEvent,
};
use chrono::{DateTime, Utc};
use hoister_shared::alerts::AlertState;
use hoister_shared::wire::ContainerMetricSample;
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;

pub trait AlertsRepository: Send + Sync + 'static + Clone {
    fn list_rules(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<AlertRule>, AlertRuleError>> + Send;

    fn create_rule(
        &self,
        user_id: &str,
        req: CreateAlertRuleRequest,
    ) -> impl Future<Output = Result<AlertRule, AlertRuleError>> + Send;

    fn delete_rule(
        &self,
        user_id: &str,
        rule_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, AlertRuleError>> + Send;

    fn set_enabled(
        &self,
        user_id: &str,
        rule_id: uuid::Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<bool, AlertRuleError>> + Send;

    /// Load the persisted firing states of `rule_ids` for one (host, project),
    /// keyed by (rule, service). Missing pairs are simply absent (state `Ok`).
    fn get_states(
        &self,
        rule_ids: &[uuid::Uuid],
        hostname: &HostName,
        project: &ProjectName,
    ) -> impl Future<Output = Result<HashMap<(uuid::Uuid, ServiceName), AlertState>, AlertRuleError>>
    + Send;

    /// Upsert the given (rule, service) states for one (host, project).
    fn put_states(
        &self,
        hostname: &HostName,
        project: &ProjectName,
        states: &[(uuid::Uuid, ServiceName, AlertState)],
    ) -> impl Future<Output = Result<(), AlertRuleError>> + Send;

    /// Append transitions to the user's alert history, pruning it back to
    /// `MAX_HISTORY_EVENTS`.
    fn record_events(
        &self,
        user_id: &str,
        events: &[NewAlertEvent],
    ) -> impl Future<Output = Result<(), AlertRuleError>> + Send;

    /// The user's recorded transitions, newest first, at most `limit` of them.
    fn list_events(
        &self,
        user_id: &str,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AlertEventRecord>, AlertRuleError>> + Send;

    /// Note that `session_id` is looking at the alert history at `now` and
    /// return the start of the *previous* session — the cutoff for "new since
    /// the last login". Rotating only when the session id actually changes is
    /// what keeps that cutoff stable while the user browses around within one
    /// login. `None` when this is the user's first recorded session.
    fn mark_seen(
        &self,
        user_id: &str,
        session_id: &str,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<DateTime<Utc>>, AlertRuleError>> + Send;
}

pub trait AlertsService: Send + Sync + 'static + Clone {
    fn list_rules(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<AlertRule>, AlertRuleError>> + Send;

    fn create_rule(
        &self,
        user_id: &str,
        req: CreateAlertRuleRequest,
    ) -> impl Future<Output = Result<AlertRule, AlertRuleError>> + Send;

    fn delete_rule(
        &self,
        user_id: &str,
        rule_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, AlertRuleError>> + Send;

    fn set_enabled(
        &self,
        user_id: &str,
        rule_id: uuid::Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<bool, AlertRuleError>> + Send;

    /// Run the user's enabled rules against one freshly ingested metrics
    /// batch observed at `now`, persisting state changes and the resulting
    /// history, and returning the transitions worth notifying about. Never
    /// fails the ingestion path: repository errors are logged and yield no
    /// incidents.
    fn evaluate(
        &self,
        user_id: &str,
        hostname: &HostName,
        project: &ProjectName,
        samples: &HashMap<ServiceName, ContainerMetricSample>,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Vec<AlertIncident>> + Send;

    /// The user's alert history for the dashboard, together with the cutoff
    /// that marks which entries arrived since their previous login. Reading
    /// the history is what advances that cutoff, so `session_id` must be the
    /// caller's current login session.
    fn history(
        &self,
        user_id: &str,
        session_id: &str,
        now: DateTime<Utc>,
    ) -> impl Future<Output = Result<AlertHistory, AlertRuleError>> + Send;
}
