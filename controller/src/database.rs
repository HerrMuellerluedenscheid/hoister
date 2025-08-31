use sqlx::{FromRow, Row, SqlitePool};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(FromRow, Debug, Clone)]
pub struct Deployment {
    pub id: i64,
    pub image_digest: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Create a new database connection
    pub async fn new(database_url: &str) -> Result<Self, DbError> {
        let pool = SqlitePool::connect(database_url).await?;
        Ok(Database { pool })
    }

    /// Initialize the database with required tables
    pub async fn init(&self) -> Result<(), DbError> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS deployment (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                email TEXT NOT NULL UNIQUE,
                created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Create a new deployment
    pub async fn create_deployment(&self, name: &str, email: &str) -> Result<i64, DbError> {
        let result = sqlx::query("INSERT INTO deployment (name, email) VALUES (?, ?)")
            .bind(name)
            .bind(email)
            .execute(&self.pool)
            .await?;

        Ok(result.last_insert_rowid())
    }

    /// Get deployment by ID
    pub async fn get_deployment(&self, id: i64) -> Result<Option<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, name, email, created_at FROM deployment WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// Get deployment by email
    pub async fn get_deployment_by_email(
        &self,
        email: &str,
    ) -> Result<Option<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, name, email, created_at FROM deployment WHERE email = ?",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// Get all deployment
    pub async fn get_all_deployment(&self) -> Result<Vec<Deployment>, DbError> {
        let deployment = sqlx::query_as::<_, Deployment>(
            "SELECT id, name, email, created_at FROM deployment ORDER BY created_at DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// Update deployment
    pub async fn update_deployment(
        &self,
        id: i64,
        name: &str,
        email: &str,
    ) -> Result<bool, DbError> {
        let result = sqlx::query("UPDATE deployment SET name = ?, email = ? WHERE id = ?")
            .bind(name)
            .bind(email)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Delete deployment
    pub async fn delete_deployment(&self, id: i64) -> Result<bool, DbError> {
        let result = sqlx::query("DELETE FROM deployment WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    /// Count total deployment
    pub async fn count_deployment(&self) -> Result<i64, DbError> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM deployment")
            .fetch_one(&self.pool)
            .await?;

        Ok(row.get("count"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Example usage
    #[tokio::test]
    async fn test_this() -> Result<(), DbError> {
        // For local SQLite file
        let db = Database::new("sqlite:///tmp/sqlite.db").await?;

        // Initialize the database
        db.init().await?;

        // Create some deployment
        let deployment_id = db
            .create_deployment("Alice Johnson", "alice@example.com")
            .await?;
        println!("Created deployment with ID: {}", deployment_id);

        db.create_deployment("Bob Wilson", "bob@example.com")
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
            "alice.smith@example.com",
        )
        .await?;

        // Get updated deployment
        if let Some(deployment) = db.get_deployment(deployment_id).await? {
            println!("Updated deployment: {:?}", deployment);
        }

        println!("Total deployment: {}", db.count_deployment().await?);

        Ok(())
    }
}
