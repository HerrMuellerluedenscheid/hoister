use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;

pub use hoister_shared::wire::ContainerMetricSample;

/// A batch of resource-usage samples shipped by an agent for one
/// (host, project) at a single point in time.
pub struct AddMetricsRequest {
    /// Owning user. In hosted mode this is the Clerk user id resolved from the
    /// agent's `hst_` token; in self-hosted dev mode it falls back to "local".
    pub(crate) user_id: String,
    pub(crate) hostname: HostName,
    pub(crate) project_name: ProjectName,
    pub(crate) samples: HashMap<ServiceName, ContainerMetricSample>,
}

impl AddMetricsRequest {
    pub fn new(
        user_id: String,
        hostname: HostName,
        project_name: ProjectName,
        samples: HashMap<ServiceName, ContainerMetricSample>,
    ) -> Self {
        Self {
            user_id,
            hostname,
            project_name,
            samples,
        }
    }
}

/// A single point in a service's resource-usage time series.
#[derive(Clone, Debug)]
pub struct MetricPoint {
    pub recorded_at: DateTime<Utc>,
    pub cpu_pct: f64,
    pub mem_bytes: u64,
    pub mem_limit_bytes: u64,
    pub net_rx_bytes: u64,
    pub net_tx_bytes: u64,
    pub storage_read_bytes: u64,
    pub storage_write_bytes: u64,
}

/// The most recent sample for one container, used by the dashboard aggregate.
#[derive(Clone, Debug)]
pub struct LatestMetric {
    pub hostname: HostName,
    pub project_name: ProjectName,
    pub service_name: ServiceName,
    pub point: MetricPoint,
}

/// How long raw samples are retained. Enforced opportunistically on insert.
pub const RETENTION_DAYS: i64 = 7;
