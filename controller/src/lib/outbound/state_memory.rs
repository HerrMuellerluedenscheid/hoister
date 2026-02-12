use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState,
};
use crate::domain::container_state::port::ContainerStateRepository;
use chrono::Utc;
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
    ) -> Option<HostProjectState> {
        let state = self.state.read().await;
        state
            .get(hostname)
            .and_then(|projects| projects.get(project_name))
            .filter(|host_project| host_project.services.contains_key(service_name))
            .map(|host_project| HostProjectState {
                services: host_project
                    .services
                    .iter()
                    .filter(|(k, _)| *k == service_name)
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
                last_updated: host_project.last_updated,
            })
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
        let entry = state
            .entry(hostname)
            .or_default()
            .entry(project_name)
            .or_insert_with(|| HostProjectState {
                services: Default::default(),
                last_updated: Utc::now(),
            });
        entry.services = container_inspect_responses;
        entry.last_updated = Utc::now();
    }
}
