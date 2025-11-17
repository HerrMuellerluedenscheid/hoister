mod local_sqlite;
mod postgresql;

use crate::server::DeploymentStatus;
use serde::Serialize;
use sqlx::{FromRow};
use thiserror::Error;
use ts_rs::TS;

type Digest = String;
type UserId = String;
type ProjectId = String;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

#[derive(TS)]
#[ts(export)]
#[derive(FromRow, Debug, Clone, Serialize)]
pub struct Deployment {
    pub id: i64,
    pub digest: Digest,
    pub status: DeploymentStatus,
    pub created_at: String,
}

pub trait DataStore: Clone + Send + Sync + 'static {
    fn create_deployment(
        &self,
        digest: &str,
        status: &DeploymentStatus,
        user_id: UserId,
        project: ProjectId,
    ) -> impl Future<Output = Result<i64, DbError>> + Send;

    fn get_deployment(
        &self,
        id: i64,
        user_id: UserId,
    ) -> impl Future<Output = Result<Option<Deployment>, DbError>> + Send;

    fn get_all_deployment(&self, user_id: UserId) -> impl Future<Output = Result<Vec<Deployment>, DbError>> + Send;
}
