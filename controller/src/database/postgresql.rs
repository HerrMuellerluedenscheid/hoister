use log::{debug, info};
use sqlx::migrate::MigrateDatabase;
use sqlx::SqlitePool;
use crate::database::{DataStore, DbError, Deployment};
use crate::server::DeploymentStatus;
use crate::database::{ProjectId, UserId};

#[derive(Debug, Clone)]
pub struct Postgres {
    pool: SqlitePool,
}

impl DataStore for Postgres {
    /// Create a new deployment
    async fn create_deployment(
        &self,
        digest: &str,
        status: &DeploymentStatus,
        user_id: UserId,
        project: ProjectId,
    ) -> Result<i64, DbError> {
        if matches!(status, DeploymentStatus::NoUpdate) {
            sqlx::query("DELETE FROM deployment WHERE status = ?")
                .bind(DeploymentStatus::NoUpdate as u8)
                .execute(&self.pool)
                .await?;
            debug!("{} - {}", digest, status)
        } else {
            info!("{} - {}", digest, status)
        }

        let result = sqlx::query("INSERT INTO deployment (digest, status) VALUES (?, ?)")
            .bind(digest)
            .bind(status)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get deployment by ID
    async fn get_deployment(&self, id: i64, user_id: UserId) -> Result<Option<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, digest, status, created_at FROM deployment WHERE id = ?",
        )
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(deployment)
    }

    /// Get all deployment
    async fn get_all_deployment(&self, user_id: UserId) -> Result<Vec<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, digest, status, created_at FROM deployment ORDER BY created_at DESC",
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(deployment)
    }
}

impl Postgres {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        info!("Connecting to database: {database_url}");
        if !sqlx::Sqlite::database_exists(database_url).await? {
            sqlx::Sqlite::create_database(database_url).await?;
        }

        let pool = SqlitePool::connect(database_url).await?;
        Ok(Postgres { pool })
    }

    /// Initialize the database with required tables
    pub async fn init(&self) -> Result<(), DbError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deployment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                digest TEXT NOT NULL,
                status INTEGER NOT NULL CHECK (status IN (0, 1, 2, 3, 4, 5)),
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // Example usage
    #[tokio::test]
    async fn test_this() -> Result<(), DbError> {
        let db = Postgres::new("sqlite:///tmp/sqlite.db").await?;
        db.init().await?;

        let deployment_id = db
            .create_deployment(
                "sdfsdfsasdfasdfaosdifjoaijsdofijaosidjfoajosdfjoiajsdfoijaosdifjoasdjfoij",
                &DeploymentStatus::Pending,
            )
            .await?;
        println!("Created deployment with ID: {deployment_id}");

        db.create_deployment("Bob Wilson", &DeploymentStatus::Pending)
            .await?;

        // Get all deployment
        let deployment = db.get_all_deployment().await?;
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

