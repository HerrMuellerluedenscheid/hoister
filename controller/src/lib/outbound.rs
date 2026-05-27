pub mod notification_dispatch;
pub mod pending_updates_memory;
pub mod postgresql;
pub mod sqlite;
pub mod state_memory;

use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project,
};
use crate::domain::deployments::ports::DeploymentsRepository;
use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError};
use crate::domain::notifiers::ports::NotifierRepository;
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
    pub async fn connect(
        url: &str,
        token_pepper: Vec<u8>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if url.starts_with("postgres") {
            info!("Using PostgreSQL backend: {url}");
            let repo = Postgresql::new(url, token_pepper).await?;
            repo.migrate().await?;
            Ok(Self::Postgresql(repo))
        } else {
            info!("Using SQLite backend: {url}");
            let repo = Sqlite::new(url, token_pepper).await?;
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
        user_id: &str,
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
        user_id: &str,
    ) -> Result<Deployment, GetDeploymentError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as DeploymentsRepository>::get_deployment(db, deployment_id, user_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as DeploymentsRepository>::get_deployment(db, deployment_id, user_id)
                    .await
            }
        }
    }

    async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: &str,
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
    async fn list_tokens(&self, user_id: &str) -> Result<Vec<ApiToken>, TokenError> {
        match self {
            Self::Sqlite(db) => <Sqlite as TokenRepository>::list_tokens(db, user_id).await,
            Self::Postgresql(db) => <Postgresql as TokenRepository>::list_tokens(db, user_id).await,
        }
    }

    async fn create_token(
        &self,
        user_id: &str,
        comment: Option<String>,
    ) -> Result<ApiToken, TokenError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as TokenRepository>::create_token(db, user_id, comment).await
            }
            Self::Postgresql(db) => {
                <Postgresql as TokenRepository>::create_token(db, user_id, comment).await
            }
        }
    }

    async fn delete_token(&self, user_id: &str, token_id: i64) -> Result<bool, TokenError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as TokenRepository>::delete_token(db, user_id, token_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as TokenRepository>::delete_token(db, user_id, token_id).await
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

impl NotifierRepository for Database {
    async fn list_notifiers(&self, user_id: &str) -> Result<Vec<Notifier>, NotifierError> {
        match self {
            Self::Sqlite(db) => <Sqlite as NotifierRepository>::list_notifiers(db, user_id).await,
            Self::Postgresql(db) => {
                <Postgresql as NotifierRepository>::list_notifiers(db, user_id).await
            }
        }
    }

    async fn create_notifier(
        &self,
        user_id: &str,
        config: NotifierConfig,
    ) -> Result<Notifier, NotifierError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as NotifierRepository>::create_notifier(db, user_id, config).await
            }
            Self::Postgresql(db) => {
                <Postgresql as NotifierRepository>::create_notifier(db, user_id, config).await
            }
        }
    }

    async fn delete_notifier(
        &self,
        user_id: &str,
        notifier_id: i64,
    ) -> Result<bool, NotifierError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as NotifierRepository>::delete_notifier(db, user_id, notifier_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as NotifierRepository>::delete_notifier(db, user_id, notifier_id).await
            }
        }
    }

    async fn set_enabled(
        &self,
        user_id: &str,
        notifier_id: i64,
        enabled: bool,
    ) -> Result<bool, NotifierError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as NotifierRepository>::set_enabled(db, user_id, notifier_id, enabled).await
            }
            Self::Postgresql(db) => {
                <Postgresql as NotifierRepository>::set_enabled(db, user_id, notifier_id, enabled)
                    .await
            }
        }
    }
}
