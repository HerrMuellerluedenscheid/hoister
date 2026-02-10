use crate::domain::deployments::models::deployment::{
    CreateDeploymentError, CreateDeploymentRequest, Deployment, DeploymentId, GetDeploymentError,
    GetProjectError, Project, ProjectId,
};
use crate::domain::deployments::models::service::{Service, ServiceId};
use crate::domain::deployments::ports::DeploymentsRepository;
use hoister_shared::{DeploymentStatus, ImageDigest, ImageName, ProjectName, ServiceName};
use log::error;
use sqlx::migrate::MigrateDatabase;
use sqlx::{Error as SqlxError, Row, SqlitePool};
use tracing::{debug, info};

#[derive(Debug, Clone)]
pub struct Sqlite {
    pool: SqlitePool,
}

impl Sqlite {
    pub async fn new(database_url: &str) -> Result<Self, SqlxError> {
        info!("Connecting to database: {database_url}");
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;

        Ok(Self { pool })
    }

    /// Run embedded database migrations.
    pub async fn migrate(&self) -> Result<(), SqlxError> {
        info!("Running database migrations");
        sqlx::migrate!().run(&self.pool).await.map_err(|e| SqlxError::Migrate(Box::new(e)))?;
        Ok(())
    }

    /// Get all deployments
    pub async fn get_all_deployments(&self) -> Result<Vec<Deployment>, SqlxError> {
        let rows = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at,
                    s.name as service_name,
                    p.name as project_name
                FROM deployment d
                JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                ORDER BY d.created_at DESC
                LIMIT 50",
        )
        .fetch_all(&self.pool)
        .await?;

        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get("id")),
                digest: row.get("digest"),
                status: row.get("status"),
                service_id: row.get("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
            })
            .collect();

        Ok(deployments)
    }

    /// Upsert a project by name, returning its ID
    pub async fn upsert_project(&self, name: &ProjectName) -> Result<i64, SqlxError> {
        let result = sqlx::query(
            r#"
            INSERT INTO project (name) VALUES (?)
            ON CONFLICT(name) DO UPDATE SET name = name
            RETURNING id
            "#,
        )
        .bind(name.as_str())
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
            INSERT INTO service (project_id, name, image) VALUES (?, ?, ?)
            ON CONFLICT(project_id, name) DO UPDATE SET image = excluded.image
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

    /// Get a project by name
    pub async fn get_project(&self, project_name: &ProjectName) -> Result<Project, SqlxError> {
        let row = sqlx::query(
            r#"
            SELECT * FROM project WHERE project.name = ?
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
            SELECT * FROM service WHERE service.name = ? AND service.project_id = ?
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
        sqlx::query("DELETE FROM deployment WHERE status = ? AND service_id = ?")
            .bind(DeploymentStatus::NoUpdate as u8)
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
                    d.created_at,
                    s.name as service_name,
                    p.name as project_name
                FROM deployment d                 JOIN service s ON d.service_id = s.id
                JOIN project p ON s.project_id = p.id
                WHERE d.id = ?",
        )
        .bind(id.0)
        .fetch_one(&self.pool)
        .await?;

        let deployment = Deployment {
            id: DeploymentId(row.get("id")),
            digest: row.get("digest"),
            status: row.get("status"),
            service_id: row.get("service_id"),
            created_at: row.get("created_at"),
            service_name: ServiceName(row.get("service_name")),
            project_name: ProjectName(row.get("project_name")),
        };

        Ok(deployment)
    }

    /// Get deployment by image
    pub async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Result<Vec<Deployment>, SqlxError> {
        let project = self.get_project(project_name).await?;
        let service = self.get_service(&project, service_name).await?;
        let rows = sqlx::query(
            "SELECT
                    d.id,
                    d.digest,
                    d.status,
                    d.service_id,
                    d.created_at,
                    s.name as service_name,
                    p.name as project_name
                FROM deployment d
                    JOIN service s ON d.service_id = s.id
                    JOIN project p ON s.project_id = p.id
                WHERE service_id = ? ORDER BY d.created_at DESC LIMIT 50",
        )
        .bind(service.id.0)
        .fetch_all(&self.pool)
        .await?;
        let deployments = rows
            .iter()
            .map(|row| Deployment {
                id: DeploymentId(row.get("id")),
                digest: row.get("digest"),
                status: row.get("status"),
                service_id: row.get("service_id"),
                created_at: row.get("created_at"),
                service_name: ServiceName(row.get("service_name")),
                project_name: ProjectName(row.get("project_name")),
            })
            .collect();

        Ok(deployments)
    }

    async fn create_deployment(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        image: &ImageName,
        digest: &ImageDigest,
        status: &DeploymentStatus,
    ) -> Result<DeploymentId, SqlxError> {
        // Upsert project and service
        let project_id = self.upsert_project(project_name).await?;
        let service_id = self.upsert_service(project_id, service_name, image).await?;

        if matches!(status, DeploymentStatus::NoUpdate) {
            self.clear_last_no_update_deployment(service_id).await?;
            debug!("deleted {} - {}", digest.as_str(), status)
        }

        let result =
            sqlx::query("INSERT INTO deployment (digest, status, service_id) VALUES (?, ?, ?)")
                .bind(digest.as_str())
                .bind(status)
                .bind(service_id)
                .execute(&self.pool)
                .await
                .expect("Failed to insert deployment");

        Ok(DeploymentId(result.last_insert_rowid()))
    }
}

impl DeploymentsRepository for Sqlite {
    async fn create_deployment(
        &self,
        req: &CreateDeploymentRequest,
    ) -> Result<DeploymentId, CreateDeploymentError> {
        self.create_deployment(
            &req.project_name,
            &req.service_name,
            &req.image_name,
            &req.image_digest,
            &req.deployment_status,
        )
        .await
        .map_err(|e| {
            error!("Failed to create deployment: {:?}", e);
            CreateDeploymentError::UnknownError
        })
    }

    async fn get_all_deployments(&self) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.get_all_deployments().await.map_err(|e| {
            error!("Failed to get all deployments: {:?}", e);
            GetDeploymentError::UnknownError
        })
    }

    async fn get_deployment(
        &self,
        deployment_id: DeploymentId,
    ) -> Result<Deployment, GetDeploymentError> {
        self.get_deployment(deployment_id).await.map_err(|e| {
            error!("Failed to get all deployments: {:?}", e);
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
    ) -> Result<Vec<Deployment>, GetDeploymentError> {
        self.get_deployments_of_service(project_name, service_name)
            .await
            .map_err(|e| {
                error!(
                    "Failed to get deployments of service: {:?} {:?} | {:?}",
                    project_name, service_name, e
                );
                match e {
                    sqlx::error::Error::RowNotFound => GetDeploymentError::DeploymentNotFound,
                    _ => GetDeploymentError::UnknownError,
                }
            })
    }

    async fn get_project(&self, project_name: &ProjectName) -> Result<Project, GetProjectError> {
        self.get_project(project_name).await.map_err(|e| {
            error!("Failed to get project: {:?}", e);
            GetProjectError::UnknownError
        })
    }
}
