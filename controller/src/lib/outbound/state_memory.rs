use crate::domain::container_state::models::state::{AddContainerStateRequest, ContainerStateData};
use crate::domain::container_state::port::ContainerStateRepository;
use bollard::models::ContainerInspectResponse;
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct StateMemory {
    state: Arc<RwLock<ContainerStateData>>,
}

impl Default for StateMemory {
    fn default() -> Self {
        Self {
            state: Arc::new(RwLock::new(ContainerStateData::new())),
        }
    }
}

impl ContainerStateRepository for StateMemory {
    async fn get_container_state(
        &self,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<ContainerInspectResponse> {
        let state = self.state.read().await;
        state
            .get(hostname)
            .and_then(|projects| {
                projects
                    .get(project_name)
                    .and_then(|services| services.get(service_name))
            })
            .cloned()
    }

    async fn get_container_states(&self) -> ContainerStateData {
        let state = self.state.read().await;
        state.clone()
    }

    async fn add_container_state(&self, request: AddContainerStateRequest) {
        let hostname = request.hostname;
        let project_name = request.project_name;
        let container_inspect_responses = request.container_inspect_responses;

        let mut state = self.state.write().await;
        *state
            .entry(hostname)
            .or_default()
            .entry(project_name)
            .or_default() = container_inspect_responses;
    }
}
