use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState,
};
use hoister_shared::{HostName, ProjectName, ServiceName};

pub trait ContainerStateRepository: Send + Sync + 'static + Clone {
    fn get_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> impl Future<Output = Option<HostProjectState>> + Send;
    fn get_container_states(
        &self,
        user_id: &str,
    ) -> impl Future<Output = ContainerStateData> + Send;
    fn add_container_state(&self, req: AddContainerStateRequest)
    -> impl Future<Output = ()> + Send;
    /// Remove a single (host, project) entry for a user. Returns `true` when a
    /// row was deleted, `false` when nothing matched.
    fn delete_project(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
    ) -> impl Future<Output = bool> + Send;
}

pub trait ContainerStateService: Send + Sync + 'static + Clone {
    fn get_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> impl Future<Output = Option<HostProjectState>> + Send;
    fn get_container_states(
        &self,
        user_id: &str,
    ) -> impl Future<Output = ContainerStateData> + Send;
    fn add_container_state(&self, req: AddContainerStateRequest)
    -> impl Future<Output = ()> + Send;
    /// Remove a single (host, project) entry for a user. Returns `true` when a
    /// row was deleted, `false` when nothing matched.
    fn delete_project(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
    ) -> impl Future<Output = bool> + Send;
}
