use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState,
};
use hoister_shared::{HostName, ProjectName, ServiceName};

pub trait ContainerStateRepository: Send + Sync + 'static + Clone {
    fn get_container_state(
        &self,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> impl Future<Output = Option<HostProjectState>> + Send;
    fn get_container_states(&self) -> impl Future<Output = ContainerStateData> + Send;
    fn add_container_state(&self, req: AddContainerStateRequest)
    -> impl Future<Output = ()> + Send;
}
// An deployments services manages deployments
pub trait ContainerStateService: Send + Sync + 'static + Clone {
    fn get_container_state(
        &self,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> impl Future<Output = Option<HostProjectState>> + Send;
    fn get_container_states(&self) -> impl Future<Output = ContainerStateData> + Send;
    fn add_container_state(&self, req: AddContainerStateRequest)
    -> impl Future<Output = ()> + Send;
}
