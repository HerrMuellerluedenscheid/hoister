use crate::domain::metrics::models::{AddMetricsRequest, LatestMetric, MetricPoint};
use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};

pub trait MetricsRepository: Send + Sync + 'static + Clone {
    /// Append a batch of samples and opportunistically prune rows older than
    /// the retention window for this user.
    fn add_metrics(&self, req: AddMetricsRequest) -> impl Future<Output = ()> + Send;
    /// Time series for one service, oldest first, since the given instant.
    fn get_service_metrics(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
        since: DateTime<Utc>,
    ) -> impl Future<Output = Vec<MetricPoint>> + Send;
    /// Latest sample per container for the user (dashboard aggregate).
    fn get_latest_metrics(&self, user_id: &str) -> impl Future<Output = Vec<LatestMetric>> + Send;
}

pub trait MetricsService: Send + Sync + 'static + Clone {
    fn add_metrics(&self, req: AddMetricsRequest) -> impl Future<Output = ()> + Send;
    fn get_service_metrics(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
        since: DateTime<Utc>,
    ) -> impl Future<Output = Vec<MetricPoint>> + Send;
    fn get_latest_metrics(&self, user_id: &str) -> impl Future<Output = Vec<LatestMetric>> + Send;
}
