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
    pub(crate) hostname: HostName,
    pub(crate) project_name: ProjectName,
    pub(crate) services: HashMap<ServiceName, ServiceState>,
}

#[derive(Clone)]
pub struct HostProjectState {
    pub services: HashMap<ServiceName, ServiceState>,
    pub last_updated: DateTime<Utc>,
}

pub(crate) type ContainerStateData = HashMap<HostName, HashMap<ProjectName, HostProjectState>>;
