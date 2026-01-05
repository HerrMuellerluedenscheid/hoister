use serde::{Deserialize, Serialize};
use shared::{DeploymentStatus, ImageDigest, ImageName, ProjectName, ServiceName};
use sqlx::migrate::MigrateDatabase;
use sqlx::{FromRow, Row, SqlitePool};
use thiserror::Error;
use tracing::{debug, info};
use ts_rs::TS;

type Digest = String;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(FromRow, Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub created_at: String,
}

#[derive(FromRow, Debug, Clone, Serialize, TS)]
#[ts(export)]
pub struct Service {
    pub id: i64,
    pub project_id: i64,
    pub name: String,
    pub image: String,
    pub created_at: String,
}

#[derive(FromRow, Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export)]
pub struct Deployment {
    pub id: i64,
    pub digest: Digest,
    pub status: DeploymentStatus,
    pub service_id: i64,
    pub created_at: String,
    pub service_name: String,
    pub project_name: String,
}

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        info!("Connecting to database: {database_url}");
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Initialize the database with required tables
    pub async fn init(&self) -> Result<(), DbError> {
        info!("Initializing database");
        sqlx::query(
            r#"
        CREATE TABLE IF NOT EXISTS project (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        CREATE TABLE IF NOT EXISTS service (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            project_id INTEGER NOT NULL REFERENCES project(id),
            name TEXT NOT NULL,
            image TEXT NOT NULL,
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
            UNIQUE(project_id, name)
        );
        CREATE TABLE IF NOT EXISTS deployment (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            digest TEXT NOT NULL,
            status INTEGER NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5)),
            service_id INTEGER NOT NULL REFERENCES service(id),
            created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
        );
        "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Upsert a project by name, returning its ID
    pub async fn upsert_project(&self, name: &ProjectName) -> Result<i64, DbError> {
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
    ) -> Result<i64, DbError> {
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
    pub async fn get_project(&self, project_name: &ProjectName) -> Result<Project, DbError> {
        let result = sqlx::query_as::<_, Project>(
            r#"
            SELECT * FROM project WHERE project.name = ?
            "#,
        )
        .bind(project_name.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Get a service by name
    pub async fn get_service(
        &self,
        project: &Project,
        service_name: &ServiceName,
    ) -> Result<Service, DbError> {
        let result = sqlx::query_as::<_, Service>(
            r#"
            SELECT * FROM service WHERE service.name = ? AND service.project_id = ?
            "#,
        )
        .bind(service_name.as_str())
        .bind(project.id)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Create a new deployment
    pub async fn create_deployment(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
        image: &ImageName,
        digest: &ImageDigest,
        status: &DeploymentStatus,
    ) -> Result<i64, DbError> {
        // Upsert project and service
        let project_id = self.upsert_project(project_name).await?;
        let service_id = self.upsert_service(project_id, service_name, image).await?;

        if matches!(status, DeploymentStatus::NoUpdate) {
            sqlx::query("DELETE FROM deployment WHERE status = ? AND service_id = ?")
                .bind(DeploymentStatus::NoUpdate as u8)
                .bind(service_id)
                .execute(&self.pool)
                .await?;
            debug!("deleted {} - {}", digest.as_str(), status)
        }

        let result =
            sqlx::query("INSERT INTO deployment (digest, status, service_id) VALUES (?, ?, ?)")
                .bind(digest.as_str())
                .bind(status)
                .bind(service_id)
                .execute(&self.pool)
                .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get deployment by ID
    pub async fn get_deployment(&self, id: i64) -> Result<Option<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
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
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// Get all deployments
    pub async fn get_all_deployments(&self) -> Result<Vec<Deployment>, DbError> {
        let deployments = sqlx::query_as::<_, Deployment>(
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

        Ok(deployments)
    }

    /// Get deployment by image
    pub async fn get_deployments_of_service(
        &self,
        project_name: &ProjectName,
        service_name: &ServiceName,
    ) -> Result<Vec<Deployment>, DbError> {
        let project = self.get_project(project_name).await?;
        let service = self.get_service(&project, service_name).await?;
        let deployments = sqlx::query_as::<_, Deployment>(
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
        .bind(service.id)
        .fetch_all(&self.pool)
        .await?;

        Ok(deployments)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example usage
    #[tokio::test]
    async fn test_create_deployment() -> Result<(), DbError> {
        let db = Database::new("sqlite:///tmp/sqlite.db").await?;
        db.init().await?;

        let deployment_id = db
            .create_deployment(
                &ProjectName::new(
                    "sdfsdfsasdfasdfaosdifjoaijsdofijaosidjfoajosdfjoiajsdfoijaosdifjoasdjfoij",
                ),
                &ServiceName::new("asdfasdf"),
                &ImageName::new("demoimage"),
                &ImageDigest::new("asdfasfoip234w"),
                &DeploymentStatus::Pending,
            )
            .await?;
        println!("Created deployment with ID: {deployment_id}");

        db.create_deployment(
            &ProjectName::new("Bob Wilson"),
            &ServiceName::new("demoservice"),
            &ImageName::new("demoimage"),
            &ImageDigest::new("asdfasfoip234w"),
            &DeploymentStatus::Pending,
        )
        .await?;

        // Get all deployment
        let deployment = db.get_all_deployments().await?;
        println!("All deployment:");
        for deployment in deployment {
            println!("  {deployment:?}");
        }

        // Get updated deployment
        if let Some(deployment) = db.get_deployment(deployment_id).await? {
            println!("Updated deployment: {deployment:?}");
        }

        Ok(())
    }
}
