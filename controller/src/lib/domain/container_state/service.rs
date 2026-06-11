use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState,
};
use crate::domain::container_state::port::{ContainerStateRepository, ContainerStateService};
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
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<HostProjectState> {
        self.container_state_repository
            .get_container_state(user_id, hostname, project_name, service_name)
            .await
    }

    async fn get_container_states(&self, user_id: &str) -> ContainerStateData {
        self.container_state_repository
            .get_container_states(user_id)
            .await
    }

    async fn add_container_state(&self, req: AddContainerStateRequest) {
        self.container_state_repository
            .add_container_state(req)
            .await
    }

    async fn delete_project(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
    ) -> bool {
        self.container_state_repository
            .delete_project(user_id, hostname, project_name)
            .await
    }
}
