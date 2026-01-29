use hoister_shared::{ProjectName, ServiceName};
use crate::domain::deployments::models::deployment::{CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError};
use crate::domain::deployments::ports::{DeploymentsRepository, DeploymentsService};

#[derive(Clone)]
pub struct Service<DR: DeploymentsRepository> {
    deployments_repository: DR,
}
impl<DR: DeploymentsRepository> Service<DR> {
    pub fn new(deployments_repository: DR) -> Self {
        Self {
            deployments_repository,
        }
    }
}

impl<DR: DeploymentsRepository> DeploymentsService for Service<DR> {
    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, CreateDeploymentError> {
        self.deployments_repository.create_deployment(req).await
    }

    async fn get_all_deployments(
        &self,
    ) -> Result<Vec<Deployment>, GetDeploymentError>{
        self.deployments_repository.get_all_deployments().await
    }

    async fn get_deployment(&self, id: DeploymentId) -> Result<Deployment, GetDeploymentError> {
        self.deployments_repository.get_deployment(id).await
    }

    async fn get_deployments_of_service(&self, project_name: &ProjectName, service_name: &ServiceName) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.deployments_repository.get_deployments_of_service(project_name, service_name).await
    }
}
