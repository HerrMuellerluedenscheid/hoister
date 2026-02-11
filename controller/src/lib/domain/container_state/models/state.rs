use bollard::models::ContainerInspectResponse;
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;

pub struct AddContainerStateRequest {
    pub(crate) hostname: HostName,
    pub(crate) project_name: ProjectName,
    pub(crate) container_inspect_responses: HashMap<ServiceName, ContainerInspectResponse>,
}

pub(crate) type ContainerStateData =
    HashMap<HostName, HashMap<ProjectName, HashMap<ServiceName, ContainerInspectResponse>>>;
