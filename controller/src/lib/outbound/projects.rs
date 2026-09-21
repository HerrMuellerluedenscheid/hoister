//! `Database` dispatch for [`ProjectsRepository`], kept out of `outbound.rs`
//! only to keep that file manageable.

use super::Database;
use super::postgresql::Postgresql;
use super::sqlite::Sqlite;
use crate::domain::deployments::models::deployment::Deployment;
use crate::domain::projects::models::{
    ProjectAccess, ProjectInvitation, ProjectMember, ProjectsError,
};
use crate::domain::projects::ports::ProjectsRepository;
use hoister_shared::ServiceName;

impl ProjectsRepository for Database {
    async fn list_accessible_projects(
        &self,
        user_id: &str,
    ) -> Result<Vec<ProjectAccess>, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::list_accessible_projects(db, user_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::list_accessible_projects(db, user_id).await
            }
        }
    }

    async fn get_project_access(
        &self,
        user_id: &str,
        project_id: uuid::Uuid,
    ) -> Result<Option<ProjectAccess>, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::get_project_access(db, user_id, project_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::get_project_access(db, user_id, project_id)
                    .await
            }
        }
    }

    async fn delete_project(&self, project_id: uuid::Uuid) -> Result<bool, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::delete_project(db, project_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::delete_project(db, project_id).await
            }
        }
    }

    async fn list_members(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Vec<ProjectMember>, ProjectsError> {
        match self {
            Self::Sqlite(db) => <Sqlite as ProjectsRepository>::list_members(db, project_id).await,
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::list_members(db, project_id).await
            }
        }
    }

    async fn add_member(
        &self,
        project_id: uuid::Uuid,
        user_id: &str,
        invited_by: &str,
    ) -> Result<bool, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::add_member(db, project_id, user_id, invited_by)
                    .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::add_member(db, project_id, user_id, invited_by)
                    .await
            }
        }
    }

    async fn remove_member(
        &self,
        project_id: uuid::Uuid,
        user_id: &str,
    ) -> Result<bool, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::remove_member(db, project_id, user_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::remove_member(db, project_id, user_id).await
            }
        }
    }

    async fn list_invitations(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Vec<ProjectInvitation>, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::list_invitations(db, project_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::list_invitations(db, project_id).await
            }
        }
    }

    async fn create_invitation(
        &self,
        project_id: uuid::Uuid,
        email: &str,
        invited_by: &str,
    ) -> Result<(ProjectInvitation, bool), ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::create_invitation(db, project_id, email, invited_by)
                    .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::create_invitation(
                    db, project_id, email, invited_by,
                )
                .await
            }
        }
    }

    async fn delete_invitation(
        &self,
        project_id: uuid::Uuid,
        invitation_id: uuid::Uuid,
    ) -> Result<bool, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::delete_invitation(db, project_id, invitation_id)
                    .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::delete_invitation(db, project_id, invitation_id)
                    .await
            }
        }
    }

    async fn delete_invitation_by_email(
        &self,
        project_id: uuid::Uuid,
        email: &str,
    ) -> Result<(), ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::delete_invitation_by_email(db, project_id, email)
                    .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::delete_invitation_by_email(
                    db, project_id, email,
                )
                .await
            }
        }
    }

    async fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> Result<Vec<uuid::Uuid>, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::claim_invitations(db, user_id, emails).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::claim_invitations(db, user_id, emails).await
            }
        }
    }

    async fn get_deployments(
        &self,
        project_id: uuid::Uuid,
        service: Option<&ServiceName>,
        limit: i64,
    ) -> Result<Vec<Deployment>, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::get_deployments(db, project_id, service, limit)
                    .await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::get_deployments(db, project_id, service, limit)
                    .await
            }
        }
    }

    async fn get_latest_rollout(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Option<Deployment>, ProjectsError> {
        match self {
            Self::Sqlite(db) => {
                <Sqlite as ProjectsRepository>::get_latest_rollout(db, project_id).await
            }
            Self::Postgresql(db) => {
                <Postgresql as ProjectsRepository>::get_latest_rollout(db, project_id).await
            }
        }
    }
}
