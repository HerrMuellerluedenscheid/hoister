use hoister_shared::{
    CreateDeployment, DeploymentStatus, ImageDigest, ImageName, ProjectName, ServiceName,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use thiserror::Error;
use ts_rs::TS;

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, TS, Type)]
pub struct DeploymentId(pub i64);

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, TS, Type)]
pub struct ProjectId(pub i64);

pub struct CreateDeploymentRequest {
    pub project_name: ProjectName,
    pub service_name: ServiceName,
    pub image_name: ImageName,
    pub image_digest: ImageDigest,
    pub deployment_status: DeploymentStatus,
}

impl From<CreateDeployment> for CreateDeploymentRequest {
    fn from(val: CreateDeployment) -> Self {
        Self {
            image_name: val.image,
            image_digest: val.digest,
            service_name: val.service,
            project_name: val.project,
            deployment_status: val.status,
        }
    }
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Deployment {
    pub id: DeploymentId,
    pub digest: String,
    pub status: DeploymentStatus,
    pub service_id: i64,
    pub created_at: String,
    pub service_name: ServiceName,
    pub project_name: ProjectName,
}

#[derive(Debug, Error)]
pub enum CreateDeploymentError {
    #[error("Unknown error")]
    UnknownError,
}

#[derive(Debug, Error)]
pub enum GetDeploymentError {
    #[error("Unknown error")]
    UnknownError,
    #[error("Deployment not found")]
    DeploymentNotFound,
}

#[derive(Debug, Error)]
pub enum GetProjectError {
    #[error("Unknown error")]
    UnknownError,
}

#[allow(dead_code)]
pub struct Project {
    pub(crate) id: ProjectId,
    pub(crate) name: ProjectName,
    pub(crate) created_at: String,
}
