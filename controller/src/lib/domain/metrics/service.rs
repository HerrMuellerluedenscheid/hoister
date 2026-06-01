use crate::domain::metrics::models::{AddMetricsRequest, LatestMetric, MetricPoint};
use crate::domain::metrics::port::{MetricsRepository, MetricsService};
use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};

#[derive(Clone)]
pub struct Service<MR: MetricsRepository> {
    metrics_repository: MR,
}

impl<MR: MetricsRepository> Service<MR> {
    pub fn new(metrics_repository: MR) -> Self {
        Self { metrics_repository }
    }
}

impl<MR: MetricsRepository> MetricsService for Service<MR> {
    async fn add_metrics(&self, req: AddMetricsRequest) {
        self.metrics_repository.add_metrics(req).await
    }

    async fn get_service_metrics(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
        since: DateTime<Utc>,
    ) -> Vec<MetricPoint> {
        self.metrics_repository
            .get_service_metrics(user_id, hostname, project_name, service_name, since)
            .await
    }

    async fn get_latest_metrics(&self, user_id: &str) -> Vec<LatestMetric> {
        self.metrics_repository.get_latest_metrics(user_id).await
    }
}
