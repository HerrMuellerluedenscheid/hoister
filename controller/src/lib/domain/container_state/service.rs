use crate::domain::container_state::models::state::{AddContainerStateRequest, ContainerStateData};
use crate::domain::container_state::port::{ContainerStateRepository, ContainerStateService};
use bollard::models::ContainerInspectResponse;
use hoister_shared::{HostName, ProjectName, ServiceName};

#[derive(Clone)]
pub struct Service<CR: ContainerStateRepository> {
    container_state_repository: CR,
}

impl<CR: ContainerStateRepository> Service<CR> {
    pub fn new(container_state_repository: CR) -> Self {
        Self {
            container_state_repository,
        }
    }
}

impl<CR: ContainerStateRepository> ContainerStateService for Service<CR> {
    async fn get_container_state(
        &self,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<ContainerInspectResponse> {
        self.container_state_repository
            .get_container_state(hostname, project_name, service_name)
            .await
    }

    async fn get_container_states(&self) -> ContainerStateData {
        self.container_state_repository.get_container_states().await
    }

    async fn add_container_state(&self, req: AddContainerStateRequest) -> () {
        self.container_state_repository
            .add_container_state(req)
            .await
    }
}
