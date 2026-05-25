use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project, ProjectId,
};
use crate::domain::deployments::models::service::{Service, ServiceId};
use crate::domain::deployments::ports::DeploymentsRepository;
use crate::domain::tokens::models::{ApiToken, TokenError};
use crate::domain::tokens::ports::TokenRepository;
use hoister_shared::{DeploymentStatus, HostName, ImageName, ProjectName, ServiceName};
use log::error;
use sqlx::{Error as SqlxError, PgPool, Row};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct Postgresql {
    pool: PgPool,
}

fn status_from_i16(val: i16) -> DeploymentStatus {
    match val {
        0 => DeploymentStatus::Pending,
        1 => DeploymentStatus::Started,
        2 => DeploymentStatus::Success,
        3 => DeploymentStatus::RollbackFinished,
        4 => DeploymentStatus::NoUpdate,
        5 => DeploymentStatus::Failed,
        6 => DeploymentStatus::TestMessage,
        _ => DeploymentStatus::Pending,
    }
}

impl Postgresql {
    pub async fn new(database_url: &str) -> Result<Self, SqlxError> {
        info!("Connecting to database: {database_url}");
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }

    /// Run embedded database migrations.
    pub async fn migrate(&self) -> Result<(), SqlxError> {
        info!("Running database migrations");
        sqlx::migrate!("migrations/postgres")
            .run(&self.pool)
            .await
            .map_err(|e| SqlxError::Migrate(Box::new(e)))?;
        Ok(())
    }

    /// Get all deployments, optionally scoped to a Clerk user.
    pub async fn get_all_deployments(
        &self,
        user_id: Option<&str>,
    ) -> Result<Vec<Deployment>, SqlxError> {
        let rows = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at::text as created_at,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                LEFT JOIN host h ON d.host_id = h.id
                WHERE ($1 IS NULL OR p.user_id = $1)
                ORDER BY d.created_at DESC
                LIMIT 50",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get("id")),
                digest: row.get("digest"),
                status: status_from_i16(row.get("status")),
                service_id: row.get("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
                hostname: HostName::new(row.get::<String, _>("hostname")),
            })
            .collect();

        Ok(deployments)
    }

    /// Upsert a project by name, returning its ID.
    /// Sets user_id only on insert; existing projects keep their user_id.
    pub async fn upsert_project(
        &self,
        name: &ProjectName,
        user_id: Option<&str>,
    ) -> Result<i64, SqlxError> {
        let result = sqlx::query(
            r#"
            INSERT INTO project (name, user_id) VALUES ($1, $2)
            ON CONFLICT(name) DO UPDATE SET name = EXCLUDED.name
            RETURNING id
            "#,
        )
        .bind(name.as_str())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Upsert a service, returning its ID
    pub async fn upsert_service(
        &self,
        project_id: i64,
        name: &ServiceName,
        image: &ImageName,
    ) -> Result<i64, SqlxError> {
        let result = sqlx::query(
            r#"
            INSERT INTO service (project_id, name, image) VALUES ($1, $2, $3)
            ON CONFLICT(project_id, name) DO UPDATE SET image = EXCLUDED.image
            RETURNING id
            "#,
        )
        .bind(project_id)
        .bind(name.as_str())
        .bind(image.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Upsert a host by hostname, returning its UUID bytes.
    /// Sets user_id only on insert; existing hosts keep their user_id.
    pub async fn upsert_host(
        &self,
        hostname: &HostName,
        user_id: Option<&str>,
    ) -> Result<Vec<u8>, SqlxError> {
        let id = uuid::Uuid::new_v4().as_bytes().to_vec();
        let result = sqlx::query(
            r#"
            INSERT INTO host (id, hostname, user_id) VALUES ($1, $2, $3)
            ON CONFLICT(hostname) DO UPDATE SET hostname = EXCLUDED.hostname
            RETURNING id
            "#,
        )
        .bind(&id)
        .bind(hostname.as_str())
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result.get("id"))
    }

    /// Get a project by name
    pub async fn get_project(&self, project_name: &ProjectName) -> Result<Project, SqlxError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, created_at::text as created_at FROM project WHERE project.name = $1
            "#,
        )
        .bind(project_name.as_str())
        .fetch_one(&self.pool)
        .await?;

        let project = Project {
            id: ProjectId(row.get("id")),
            name: ProjectName(row.get("name")),
            created_at: row.get("created_at"),
        };
        Ok(project)
    }

    pub async fn get_service(
        &self,
        project: &Project,
        service_name: &ServiceName,
    ) -> Result<Service, SqlxError> {
        let row = sqlx::query(
            r#"
            SELECT id, name, project_id, created_at::text as created_at
            FROM service WHERE service.name = $1 AND service.project_id = $2
            "#,
        )
        .bind(service_name.as_str())
        .bind(project.id.0)
        .fetch_one(&self.pool)
        .await?;

        let result = Service {
            id: ServiceId(row.get("id")),
            name: ServiceName(row.get("name")),
            project_id: ProjectId(row.get("project_id")),
            created_at: row.get("created_at"),
        };
        Ok(result)
    }

    async fn clear_last_no_update_deployment(&self, service_id: i64) -> Result<(), SqlxError> {
        sqlx::query("DELETE FROM deployment WHERE status = $1 AND service_id = $2")
            .bind(DeploymentStatus::NoUpdate as i16)
            .bind(service_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// Get deployment by ID
    pub async fn get_deployment(&self, id: DeploymentId) -> Result<Deployment, SqlxError> {
        let row = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at::text as created_at,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                LEFT JOIN host h ON d.host_id = h.id
                WHERE d.id = $1",
        )
        .bind(id.0)
        .fetch_one(&self.pool)
        .await?;

        let deployment = Deployment {
            id: DeploymentId(row.get("id")),
            digest: row.get("digest"),
            status: status_from_i16(row.get("status")),
            service_id: row.get("service_id"),
            created_at: row.get("created_at"),
            service_name: ServiceName(row.get("service_name")),
            project_name: ProjectName(row.get("project_name")),
            hostname: HostName::new(row.get::<String, _>("hostname")),
        };

        Ok(deployment)
    }

    /// Get deployments by service, optionally scoped to a Clerk user.
    pub async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: Option<&str>,
    ) -> Result<Vec<Deployment>, SqlxError> {
        let project = self.get_project(project_name).await?;
        let service = self.get_service(&project, service_name).await?;
        let rows = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at::text as created_at,
                    s.name as service_name,
                    p.name as project_name,
                    COALESCE(h.hostname, 'unknown') as hostname
                FROM deployment d
                    JOIN service s ON d.service_id = s.id
                    JOIN project p ON s.project_id = p.id
                    LEFT JOIN host h ON d.host_id = h.id
                WHERE d.service_id = $1 AND ($2 IS NULL OR p.user_id = $2)
                ORDER BY d.created_at DESC LIMIT 50",
        )
        .bind(service.id.0)
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get("id")),
                digest: row.get("digest"),
                status: status_from_i16(row.get("status")),
                service_id: row.get("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
                hostname: HostName::new(row.get::<String, _>("hostname")),
            })
            .collect();

        Ok(deployments)
    }

    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, SqlxError> {
        let user_id = req.user_id.as_deref();
        let project_id = self.upsert_project(&req.project_name, user_id).await?;
        let service_id = self
            .upsert_service(project_id, &req.service_name, &req.image_name)
            .await?;
        let host_id = self.upsert_host(&req.hostname, user_id).await?;

        if matches!(req.deployment_status, DeploymentStatus::NoUpdate) {
            self.clear_last_no_update_deployment(service_id).await?;
            debug!(
                "deleted {} - {}",
                req.image_digest.as_str(),
                req.deployment_status
            )
        }

        let result = sqlx::query(
            "INSERT INTO deployment (digest, status, service_id, host_id) VALUES ($1, $2, $3, $4) RETURNING id",
        )
        .bind(req.image_digest.as_str())
        .bind(req.deployment_status.clone() as i16)
        .bind(service_id)
        .bind(&host_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(DeploymentId(result.get("id")))
    }
}

impl TokenRepository for Postgresql {
    async fn get_or_create_token(&self, user_id: &str) -> Result<ApiToken, TokenError> {
        let already_exists: Option<i64> =
            sqlx::query_scalar("SELECT id FROM api_token WHERE user_id = $1")
                .bind(user_id)
                .fetch_optional(&self.pool)
                .await
                .map_err(|_| TokenError::UnknownError)?;

        if already_exists.is_some() {
            return Ok(ApiToken {
                token: None,
                user_id: user_id.to_string(),
                is_new: false,
            });
        }

        let token = format!("hst_{}", uuid::Uuid::new_v4().simple());
        let token_hash = crate::domain::tokens::hash::hash_token(&token);
        sqlx::query("INSERT INTO api_token (token_hash, user_id) VALUES ($1, $2)")
            .bind(&token_hash)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(|_| TokenError::UnknownError)?;

        Ok(ApiToken {
            token: Some(token),
            user_id: user_id.to_string(),
            is_new: true,
        })
    }

    async fn find_user_by_token(&self, token: &str) -> Option<String> {
        let token_hash = crate::domain::tokens::hash::hash_token(token);
        sqlx::query_scalar::<_, String>("SELECT user_id FROM api_token WHERE token_hash = $1")
            .bind(token_hash)
            .fetch_optional(&self.pool)
            .await
            .ok()
            .flatten()
    }
}

impl DeploymentsRepository for Postgresql {
    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, CreateDeploymentError> {
        self.create_deployment(req).await.map_err(|e| {
            error!("Failed to create deployment: {e:?}");
            CreateDeploymentError::UnknownError
        })
    }

    async fn get_all_deployments(
        &self,
        user_id: Option<&str>,
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.get_all_deployments(user_id).await.map_err(|e| {
            error!("Failed to get all deployments: {e:?}");
            GetDeploymentError::UnknownError
        })
    }

    async fn get_deployment(
        &self,
        deployment_id: DeploymentId,
    ) -> Result<Deployment, GetDeploymentError> {
        self.get_deployment(deployment_id).await.map_err(|e| {
            error!("Failed to get deployment: {e:?}");
            match e {
                sqlx::error::Error::RowNotFound => GetDeploymentError::DeploymentNotFound,
                _ => GetDeploymentError::UnknownError,
            }
        })
    }

    async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        user_id: Option<&str>,
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.get_deployments_of_service(project_name, service_name, user_id)
            .await
            .map_err(|e| match e {
                sqlx::error::Error::RowNotFound => GetDeploymentError::DeploymentNotFound,
                _ => {
                    error!(
                        "Failed to get deployments of service: {project_name:?} {service_name:?} | {e:?}"
                    );
                    GetDeploymentError::UnknownError
                }
            })
    }

    async fn get_project(&self, project_name: &ProjectName) -> Result<Project, GetProjectError> {
        self.get_project(project_name).await.map_err(|e| {
            error!("Failed to get project: {e:?}");
            GetProjectError::UnknownError
        })
    }
}
