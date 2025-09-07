use crate::server::DeploymentStatus;
use serde::Serialize;
use sqlx::migrate::MigrateDatabase;
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;

type Digest = String;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(FromRow, Debug, Clone, Serialize)]
pub struct Deployment {
    pub id: i64,
    pub digest: Digest,
    pub status: DeploymentStatus,
    pub created_at: String,
}

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Initialize the database with required tables
    pub async fn init(&self) -> Result<(), DbError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deployment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                digest TEXT NOT NULL,
                status INTEGER NOT NULL CHECK (status IN (0, 1, 2, 3)),
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a new deployment
    pub async fn create_deployment(
        &self,
        digest: &str,
        status: &DeploymentStatus,
    ) -> Result<i64, DbError> {
        let result = sqlx::query("INSERT INTO deployment (digest, status) VALUES (?, ?)")
            .bind(digest)
            .bind(status)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get deployment by ID
    pub async fn get_deployment(&self, id: i64) -> Result<Option<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, digest, status, created_at FROM deployment WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// Get all deployment
    pub async fn get_all_deployment(&self) -> Result<Vec<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, digest, status, created_at FROM deployment ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// Update deployment
    pub async fn update_deployment(
        &self,
        id: i64,
        digest: &str,
        status: &DeploymentStatus,
    ) -> Result<bool, DbError> {
        let result = sqlx::query("UPDATE deployment SET digest = ?, status = ? WHERE id = ?")
            .bind(digest)
            .bind(status)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example usage
    #[tokio::test]
    async fn test_this() -> Result<(), DbError> {
        let db = Database::new("sqlite:///tmp/sqlite.db").await?;
        db.init().await?;

        let deployment_id = db
            .create_deployment(
                "sdfsdfsasdfasdfaosdifjoaijsdofijaosidjfoajosdfjoiajsdfoijaosdifjoasdjfoij",
                &DeploymentStatus::Pending,
            )
            .await?;
        println!("Created deployment with ID: {}", deployment_id);

        db.create_deployment("Bob Wilson", &DeploymentStatus::Pending)
            .await?;

        // Get all deployment
        let deployment = db.get_all_deployment().await?;
        println!("All deployment:");
        for deployment in deployment {
            println!("  {:?}", deployment);
        }

        // Update a deployment
        db.update_deployment(
            deployment_id,
            "Alice Johnson-Smith",
            &DeploymentStatus::Pending,
        )
        .await?;

        // Get updated deployment
        if let Some(deployment) = db.get_deployment(deployment_id).await? {
            println!("Updated deployment: {:?}", deployment);
        }

        Ok(())
    }
}
