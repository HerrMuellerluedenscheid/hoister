use bollard::models::ContainerInspectResponse;
use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One service's worth of state shipped from the agent: the inspect payload plus
/// an optional tail of recent container logs. Logs are only attached when the
/// container is in a non-running state (restarting / exited / dead) — for the
/// happy path we keep the payload small.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceState {
    pub inspect: ContainerInspectResponse,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_logs: Option<String>,
}

pub struct AddContainerStateRequest {
    /// Owning user. In hosted mode this is the Clerk user id resolved from the
    /// agent's `hst_` token; in self-hosted dev mode it falls back to "local".
    pub(crate) user_id: String,
    pub(crate) hostname: HostName,
    pub(crate) project_name: ProjectName,
    pub(crate) services: HashMap<ServiceName, ServiceState>,
}

#[derive(Clone)]
pub struct HostProjectState {
    pub services: HashMap<ServiceName, ServiceState>,
    pub last_updated: DateTime<Utc>,
}

/// One user's view of container state. The repository stores a separate copy
/// of this per user so reads can never leak across tenants.
pub(crate) type ContainerStateData = HashMap<HostName, HashMap<ProjectName, HostProjectState>>;
