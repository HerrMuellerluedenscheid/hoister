pub mod pending_updates_memory;
pub mod postgresql;
pub mod sqlite;
pub mod state_memory;

use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project,
};
use crate::domain::deployments::ports::DeploymentsRepository;
use crate::domain::tokens::models::{ApiToken, TokenError};
use crate::domain::tokens::ports::TokenRepository;
use hoister_shared::{ProjectName, ServiceName};
use log::info;
use postgresql::Postgresql;
use sqlite::Sqlite;

/// A database connection that can be either SQLite or PostgreSQL,
/// selected at runtime based on the URL scheme.
#[derive(Clone)]
pub enum Database {
    Sqlite(Sqlite),
    Postgresql(Postgresql),
}

impl Database {
    /// Connect to the database and run migrations.
    /// URLs starting with `postgres` use PostgreSQL; everything else uses SQLite.
    pub async fn connect(url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        if url.starts_with("postgres") {
            info!("Using PostgreSQL backend: {url}");
            let repo = Postgresql::new(url).await?;
            repo.migrate().await?;
            Ok(Self::Postgresql(repo))
        } else {
            info!("Using SQLite backend: {url}");
            let repo = Sqlite::new(url).await?;
            repo.migrate().await?;
            Ok(Self::Sqlite(repo))
        }
    }
}

impl DeploymentsRepository for Database {
    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, CreateDeploymentError> {
        match self {
            Self::Sqlite(db) => <Sqlite as DeploymentsRepository>::create_deployment(db, req).await,
            Self::Postgresql(db) => {
                <Postgresql as DeploymentsRepository>::create_deployment(db, req).await
            }
        }
    }

    async fn get_all_deployments(
        &self,
        user_id: Option<&str>,
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as DeploymentsRepository>::get_all_deployments(db, user_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as DeploymentsRepository>::get_all_deployments(db, user_id).await
            }
        }
    }

    async fn get_deployment(
        &self,
        deployment_id: DeploymentId,
    ) -> Result<Deployment, GetDeploymentError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as DeploymentsRepository>::get_deployment(db, deployment_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as DeploymentsRepository>::get_deployment(db, deployment_id).await
            }
        }
    }

    async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: Option<&str>,
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as DeploymentsRepository>::get_deployments_of_service(
                    db,
                    project_name,
                    service_name,
                    user_id,
                )
                .await
            }
            Self::Postgresql(db) => {
                <Postgresql as DeploymentsRepository>::get_deployments_of_service(
                    db,
                    project_name,
                    service_name,
                    user_id,
                )
                .await
            }
        }
    }

    async fn get_project(&self, project_name: &ProjectName) -> Result<Project, GetProjectError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as DeploymentsRepository>::get_project(db, project_name).await
            }
            Self::Postgresql(db) => {
                <Postgresql as DeploymentsRepository>::get_project(db, project_name).await
            }
        }
    }
}

impl TokenRepository for Database {
    async fn get_or_create_token(&self, user_id: &str) -> Result<ApiToken, TokenError> {
        match self {
            Self::Sqlite(db) => <Sqlite as TokenRepository>::get_or_create_token(db, user_id).await,
            Self::Postgresql(db) => {
                <Postgresql as TokenRepository>::get_or_create_token(db, user_id).await
            }
        }
    }

    async fn find_user_by_token(&self, token: &str) -> Option<String> {
        match self {
            Self::Sqlite(db) => <Sqlite as TokenRepository>::find_user_by_token(db, token).await,
            Self::Postgresql(db) => {
                <Postgresql as TokenRepository>::find_user_by_token(db, token).await
            }
        }
    }
}
