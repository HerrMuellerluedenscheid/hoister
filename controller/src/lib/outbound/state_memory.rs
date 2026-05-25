use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState,
};
use crate::domain::container_state::port::ContainerStateRepository;
use chrono::Utc;
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory container state, partitioned by user. Every read and write goes
/// through a user_id key so two users with the same project/service names
/// cannot see each other's containers.
#[derive(Clone, Default)]
pub struct StateMemory {
    state: Arc<RwLock<HashMap<String, ContainerStateData>>>,
}

impl ContainerStateRepository for StateMemory {
    async fn get_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<HostProjectState> {
        let state = self.state.read().await;
        state
            .get(user_id)
            .and_then(|data| data.get(hostname))
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

    async fn get_container_states(&self, user_id: &str) -> ContainerStateData {
        let state = self.state.read().await;
        state.get(user_id).cloned().unwrap_or_default()
    }

    async fn add_container_state(&self, request: AddContainerStateRequest) {
        let AddContainerStateRequest {
            user_id,
            hostname,
            project_name,
            services,
        } = request;

        let mut state = self.state.write().await;
        let user_data = state.entry(user_id).or_default();
        let entry = user_data
            .entry(hostname)
            .or_default()
            .entry(project_name)
            .or_insert_with(|| HostProjectState {
                services: Default::default(),
                last_updated: Utc::now(),
            });
        entry.services = services;
        entry.last_updated = Utc::now();
    }
}
