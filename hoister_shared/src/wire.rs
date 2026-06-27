//! Wire-format types shared between the (MIT) agent and the (AGPL)
//! controller. Live here so the agent can depend on these without linking
//! the controller crate.

use crate::{ContainerID, HostName, ProjectName, ServiceName};
use bollard::models::ContainerInspectResponse;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One service's worth of state shipped from the agent: the inspect payload
/// plus an optional tail of recent container logs. Logs are only attached
/// when the container is in a non-running state (restarting / exited /
/// dead) — for the happy path we keep the payload small.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceState {
    pub inspect: ContainerInspectResponse,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_logs: Option<String>,
}

/// Body of POST /container/state/{hostname}/{project_name}.
#[derive(Serialize, Deserialize, Debug)]
pub struct PostContainerStateRequest {
    pub project_name: ProjectName,
    pub payload: HashMap<ServiceName, ServiceState>,
}

/// A single resource-usage sample for one service, captured by the agent
/// from Docker's `stats` endpoint. Only shipped when the operator opts in
/// via `HOISTER_REPORT_METRICS=true`.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainerMetricSample {
    /// CPU usage as a percentage. Ranges 0..(100 * number_of_cpus), matching
    /// the figure `docker stats` reports.
    pub cpu_pct: f64,
    /// Current memory usage in bytes (page cache subtracted when available,
    /// so it reflects RSS-like usage like `docker stats`).
    pub mem_bytes: u64,
    /// Memory limit in bytes. `0` means unlimited.
    pub mem_limit_bytes: u64,
    /// Total bytes received across all network interfaces since container start.
    #[serde(default)]
    pub net_rx_bytes: u64,
    /// Total bytes transmitted across all network interfaces since container start.
    #[serde(default)]
    pub net_tx_bytes: u64,
    /// Total bytes read from disk since container start (Linux block I/O via
    /// cgroup `blkio_stats` — the "BLOCK I/O" figure in `docker stats`).
    #[serde(default)]
    pub disk_read_bytes: u64,
    /// Total bytes written to disk since container start (Linux block I/O).
    #[serde(default)]
    pub disk_write_bytes: u64,
}

/// Body of POST /container/metrics/{hostname}/{project_name}.
#[derive(Serialize, Deserialize, Debug)]
pub struct PostContainerMetricsRequest {
    pub project_name: ProjectName,
    pub payload: HashMap<ServiceName, ContainerMetricSample>,
}

/// Body of POST /container/logs/{hostname}/{project_name}/{service_name}.
/// A redacted log tail forwarded by the agent in response to a
/// `ControllerEvent::RequestLogs`. The controller holds this in memory only —
/// it is never written to the database.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PostContainerLogsRequest {
    pub logs: String,
}

/// SSE events the controller broadcasts to subscribed agents.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ControllerEvent {
    Retry((ProjectName, ContainerID)),
    ApplyUpdate((HostName, ProjectName, ServiceName)),
    /// On-demand log request: ask the agent on `HostName` to ship the current
    /// log tail for one service. Honoured only when that agent was started with
    /// `HOISTER_REPORT_LOGS=true`; otherwise the agent ignores it.
    RequestLogs((HostName, ProjectName, ServiceName)),
}
