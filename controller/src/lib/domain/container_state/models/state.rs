use bollard::models::ContainerInspectResponse;
use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;

pub struct AddContainerStateRequest {
    pub(crate) hostname: HostName,
    pub(crate) project_name: ProjectName,
    pub(crate) container_inspect_responses: HashMap<ServiceName, ContainerInspectResponse>,
}

#[derive(Clone)]
pub struct HostProjectState {
    pub services: HashMap<ServiceName, ContainerInspectResponse>,
    pub last_updated: DateTime<Utc>,
}

pub(crate) type ContainerStateData = HashMap<HostName, HashMap<ProjectName, HostProjectState>>;
