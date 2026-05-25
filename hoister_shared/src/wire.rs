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

/// SSE events the controller broadcasts to subscribed agents.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum ControllerEvent {
    Retry((ProjectName, ContainerID)),
    ApplyUpdate((HostName, ProjectName, ServiceName)),
}
