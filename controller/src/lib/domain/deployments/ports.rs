use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project,
};
use hoister_shared::{ProjectName, ServiceName};

pub trait DeploymentsRepository: Send + Sync + 'static + Clone {
    fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> impl Future<Output = Result<DeploymentId, CreateDeploymentError>> + Send;

    fn get_all_deployments(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Deployment>, GetDeploymentError>> + Send;

    fn get_deployment(
        &self,
        deployment_id: DeploymentId,
        user_id: &str,
    ) -> impl Future<Output = Result<Deployment, GetDeploymentError>> + Send;

    fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Deployment>, GetDeploymentError>> + Send;

    fn get_project(
        &self,
        project_name: &ProjectName,
    ) -> impl Future<Output = Result<Project, GetProjectError>> + Send;
}

pub trait DeploymentsService: Send + Sync + 'static + Clone {
    fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> impl Future<Output = Result<DeploymentId, CreateDeploymentError>> + Send;

    fn get_all_deployments(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Deployment>, GetDeploymentError>> + Send;

    fn get_deployment(
        &self,
        id: DeploymentId,
        user_id: &str,
    ) -> impl Future<Output = Result<Deployment, GetDeploymentError>> + Send;

    fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Deployment>, GetDeploymentError>> + Send;
}
