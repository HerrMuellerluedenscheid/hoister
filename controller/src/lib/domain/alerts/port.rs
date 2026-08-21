use crate::domain::alerts::models::{
    AlertIncident, AlertRule, AlertRuleError, CreateAlertRuleRequest,
};
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
    /// batch (unix-seconds `now`), persisting state changes and returning the
    /// transitions worth notifying about. Never fails the ingestion path:
    /// repository errors are logged and yield no incidents.
    fn evaluate(
        &self,
        user_id: &str,
        hostname: &HostName,
        project: &ProjectName,
        samples: &HashMap<ServiceName, ContainerMetricSample>,
        now: i64,
    ) -> impl Future<Output = Vec<AlertIncident>> + Send;
}
