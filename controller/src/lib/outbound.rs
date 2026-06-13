pub mod notification_dispatch;
pub mod pending_updates_memory;
pub mod postgresql;
pub mod secrets;
pub mod sqlite;
pub mod state_memory;

use crate::domain::billing::models::{Plan, PlanError};
use crate::domain::billing::ports::PlanRepository;
use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, HostProjectState,
};
use crate::domain::container_state::port::ContainerStateRepository;
use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project,
};
use crate::domain::deployments::ports::DeploymentsRepository;
use crate::domain::metrics::models::{AddMetricsRequest, LatestMetric, MetricPoint};
use crate::domain::metrics::port::MetricsRepository;
use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError};
use crate::domain::notifiers::ports::NotifierRepository;
use crate::domain::tokens::models::{ApiToken, TokenError};
use crate::domain::tokens::ports::TokenRepository;
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::info;
use postgresql::Postgresql;
use sqlite::Sqlite;

/// Mask the password in a database URL before logging it (CWE-532). Postgres
/// URLs carry the password inline (`postgres://user:pass@host/db`); sqlite URLs
/// are file paths with no password and pass through unchanged.
pub(crate) fn redact_db_url(url: &str) -> String {
    match url::Url::parse(url) {
        Ok(mut parsed) if parsed.password().is_some() => {
            let _ = parsed.set_password(Some("***"));
            parsed.to_string()
        }
        Ok(_) => url.to_string(),
        Err(_) => "<unparseable database url>".to_string(),
    }
}

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
        aead: crate::outbound::secrets::Aead,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        if url.starts_with("postgres") {
            info!("Using PostgreSQL backend: {}", redact_db_url(url));
            let repo = Postgresql::new(url, token_pepper, aead).await?;
            repo.migrate().await?;
            Ok(Self::Postgresql(repo))
        } else {
            info!("Using SQLite backend: {}", redact_db_url(url));
            let repo = Sqlite::new(url, token_pepper, aead).await?;
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

    async fn delete_token(&self, user_id: &str, token_id: uuid::Uuid) -> Result<bool, TokenError> {
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
        notifier_id: uuid::Uuid,
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
        notifier_id: uuid::Uuid,
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

impl PlanRepository for Database {
    async fn get_plan(&self, user_id: &str) -> Result<Plan, PlanError> {
        match self {
            Self::Sqlite(db) => <Sqlite as PlanRepository>::get_plan(db, user_id).await,
            Self::Postgresql(db) => <Postgresql as PlanRepository>::get_plan(db, user_id).await,
        }
    }

    async fn set_plan(&self, user_id: &str, plan: Plan) -> Result<(), PlanError> {
        match self {
            Self::Sqlite(db) => <Sqlite as PlanRepository>::set_plan(db, user_id, plan).await,
            Self::Postgresql(db) => {
                <Postgresql as PlanRepository>::set_plan(db, user_id, plan).await
            }
        }
    }

    async fn upsert_user(&self, user_id: &str) {
        match self {
            Self::Sqlite(db) => <Sqlite as PlanRepository>::upsert_user(db, user_id).await,
            Self::Postgresql(db) => <Postgresql as PlanRepository>::upsert_user(db, user_id).await,
        }
    }

    async fn delete_user(&self, user_id: &str) -> bool {
        match self {
            Self::Sqlite(db) => <Sqlite as PlanRepository>::delete_user(db, user_id).await,
            Self::Postgresql(db) => <Postgresql as PlanRepository>::delete_user(db, user_id).await,
        }
    }
}

impl ContainerStateRepository for Database {
    async fn get_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Option<HostProjectState> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ContainerStateRepository>::get_container_state(
                    db,
                    user_id,
                    hostname,
                    project_name,
                    service_name,
                )
                .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ContainerStateRepository>::get_container_state(
                    db,
                    user_id,
                    hostname,
                    project_name,
                    service_name,
                )
                .await
            }
        }
    }

    async fn get_container_states(&self, user_id: &str) -> ContainerStateData {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ContainerStateRepository>::get_container_states(db, user_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ContainerStateRepository>::get_container_states(db, user_id).await
            }
        }
    }

    async fn add_container_state(&self, req: AddContainerStateRequest) {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ContainerStateRepository>::add_container_state(db, req).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ContainerStateRepository>::add_container_state(db, req).await
            }
        }
    }

    async fn delete_project(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
    ) -> bool {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ContainerStateRepository>::delete_project(
                    db,
                    user_id,
                    hostname,
                    project_name,
                )
                .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ContainerStateRepository>::delete_project(
                    db,
                    user_id,
                    hostname,
                    project_name,
                )
                .await
            }
        }
    }

    async fn touch_container_state(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
    ) {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ContainerStateRepository>::touch_container_state(
                    db,
                    user_id,
                    hostname,
                    project_name,
                )
                .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ContainerStateRepository>::touch_container_state(
                    db,
                    user_id,
                    hostname,
                    project_name,
                )
                .await
            }
        }
    }
}

impl MetricsRepository for Database {
    async fn add_metrics(&self, req: AddMetricsRequest) {
        match self {
            Self::Sqlite(db) => <Sqlite as MetricsRepository>::add_metrics(db, req).await,
            Self::Postgresql(db) => <Postgresql as MetricsRepository>::add_metrics(db, req).await,
        }
    }

    async fn get_service_metrics(
        &self,
        user_id: &str,
        hostname: &HostName,
        project_name: &ProjectName,
        service_name: &ServiceName,
        since: chrono::DateTime<chrono::Utc>,
    ) -> Vec<MetricPoint> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as MetricsRepository>::get_service_metrics(
                    db,
                    user_id,
                    hostname,
                    project_name,
                    service_name,
                    since,
                )
                .await
            }
            Self::Postgresql(db) => {
                <Postgresql as MetricsRepository>::get_service_metrics(
                    db,
                    user_id,
                    hostname,
                    project_name,
                    service_name,
                    since,
                )
                .await
            }
        }
    }

    async fn get_latest_metrics(&self, user_id: &str) -> Vec<LatestMetric> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as MetricsRepository>::get_latest_metrics(db, user_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as MetricsRepository>::get_latest_metrics(db, user_id).await
            }
        }
    }
}
