use hoister_shared::{
    CreateDeployment, DeploymentStatus, HostName, ImageDigest, ImageName, ProjectName, ServiceName,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};
use thiserror::Error;
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS, Type)]
#[sqlx(transparent)]
pub struct DeploymentId(pub uuid::Uuid);

#[derive(Debug, Clone, Serialize, Deserialize, TS, Type)]
#[sqlx(transparent)]
pub struct ProjectId(pub uuid::Uuid);

pub struct CreateDeploymentRequest {
    pub project_name: ProjectName,
    pub service_name: ServiceName,
    pub image_name: ImageName,
    pub image_digest: ImageDigest,
    pub deployment_status: DeploymentStatus,
    pub hostname: HostName,
    /// Redacted log tail of the failed container on rollback/failure.
    pub logs: Option<String>,
    /// Owning tenant. Always resolved by the auth middleware before the
    /// handler sees the request, so this is never `None`.
    pub user_id: String,
}

impl CreateDeploymentRequest {
    pub fn from_payload(payload: CreateDeployment, user_id: String) -> Self {
        Self {
            image_name: payload.image,
            image_digest: payload.digest,
            service_name: payload.service,
            project_name: payload.project,
            deployment_status: payload.status,
            hostname: payload.hostname,
            logs: payload.logs,
            user_id,
        }
    }
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Deployment {
    pub id: DeploymentId,
    pub digest: String,
    pub status: DeploymentStatus,
    pub service_id: uuid::Uuid,
    pub created_at: String,
    pub service_name: ServiceName,
    pub project_name: ProjectName,
    pub hostname: HostName,
    /// Redacted log tail captured on rollback/failure; `None` otherwise.
    pub logs: Option<String>,
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
