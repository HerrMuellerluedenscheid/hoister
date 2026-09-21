use super::Sqlite;
use crate::domain::deployments::models::deployment::{Deployment, DeploymentId};
use crate::domain::projects::models::{
    ProjectAccess, ProjectInvitation, ProjectMember, ProjectRole, ProjectsError, ReceivedInvitation,
};
use crate::domain::projects::ports::ProjectsRepository;
use hoister_shared::{DeploymentStatus, HostName, ProjectName, ServiceName};
use log::error;
use sqlx::Row;
use sqlx::sqlite::SqliteRow;

/// Projects the user owns or is a member of. `?1` is the viewing user.
const ACCESSIBLE_PROJECTS: &str = "SELECT p.id, p.name, h.hostname, p.user_id, p.created_at,
        (SELECT COUNT(*) FROM project_member pm WHERE pm.project_id = p.id) AS member_count,
        (SELECT COUNT(*) FROM notifier n WHERE n.project_id = p.id) AS notifier_count
    FROM project p
    JOIN host h ON p.host_id = h.id
    WHERE (p.user_id = ?1
        OR EXISTS (SELECT 1 FROM project_member pm WHERE pm.project_id = p.id AND pm.user_id = ?1))";

const INVITATION_COLUMNS: &str =
    "SELECT id, project_id, email, user_id, invited_by, created_at FROM project_invitation";

const DEPLOYMENT_COLUMNS: &str =
    "SELECT d.id, d.digest, d.status, d.service_id, d.created_at, d.logs,
        s.name AS service_name, p.name AS project_name,
        COALESCE(h.hostname, 'unknown') AS hostname
    FROM deployment d
    JOIN service s ON d.service_id = s.id
    JOIN project p ON s.project_id = p.id
    LEFT JOIN host h ON d.host_id = h.id";

fn db_error(what: &'static str) -> impl FnOnce(sqlx::Error) -> ProjectsError {
    move |e| {
        error!("{what} failed: {e:?}");
        ProjectsError::UnknownError
    }
}

fn project_access(row: &SqliteRow, viewer: &str) -> ProjectAccess {
    let owner_id: String = row.get("user_id");
    ProjectAccess {
        id: row.get::<uuid::Uuid, _>("id"),
        name: ProjectName::new(row.get::<String, _>("name")),
        hostname: HostName::new(row.get::<String, _>("hostname")),
        role: if owner_id == viewer {
            ProjectRole::Owner
        } else {
            ProjectRole::Member
        },
        owner_id,
        created_at: row.get("created_at"),
        member_count: row.get("member_count"),
        notifier_count: row.get("notifier_count"),
    }
}

fn invitation(row: &SqliteRow) -> ProjectInvitation {
    ProjectInvitation {
        id: row.get::<uuid::Uuid, _>("id"),
        project_id: row.get::<uuid::Uuid, _>("project_id"),
        email: row.get("email"),
        user_id: row.get("user_id"),
        invited_by: row.get("invited_by"),
        created_at: row.get("created_at"),
    }
}

fn deployment(row: &SqliteRow) -> Deployment {
    Deployment {
        id: DeploymentId(row.get::<uuid::Uuid, _>("id")),
        digest: row.get("digest"),
        status: row.get("status"),
        service_id: row.get::<uuid::Uuid, _>("service_id"),
        created_at: row.get("created_at"),
        service_name: ServiceName(row.get("service_name")),
        project_name: ProjectName(row.get("project_name")),
        hostname: HostName::new(row.get::<String, _>("hostname")),
        logs: row.get("logs"),
    }
}

impl ProjectsRepository for Sqlite {
    async fn list_accessible_projects(
        &self,
        user_id: &str,
    ) -> Result<Vec<ProjectAccess>, ProjectsError> {
        let sql = format!("{ACCESSIBLE_PROJECTS} ORDER BY p.name, h.hostname");
        let rows = sqlx::query(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error("list_accessible_projects"))?;
        Ok(rows.iter().map(|r| project_access(r, user_id)).collect())
    }

    async fn get_project_access(
        &self,
        user_id: &str,
        project_id: uuid::Uuid,
    ) -> Result<Option<ProjectAccess>, ProjectsError> {
        let sql = format!("{ACCESSIBLE_PROJECTS} AND p.id = ?2");
        let row = sqlx::query(&sql)
            .bind(user_id)
            .bind(project_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_error("get_project_access"))?;
        Ok(row.map(|r| project_access(&r, user_id)))
    }

    async fn delete_project(&self, project_id: uuid::Uuid) -> Result<bool, ProjectsError> {
        let result = sqlx::query("DELETE FROM project WHERE id = ?")
            .bind(project_id)
            .execute(&self.pool)
            .await
            .map_err(db_error("delete_project"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_members(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Vec<ProjectMember>, ProjectsError> {
        let rows = sqlx::query(
            "SELECT user_id, invited_by, created_at FROM project_member
                WHERE project_id = ?
                ORDER BY created_at, user_id",
        )
        .bind(project_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error("list_members"))?;
        Ok(rows
            .iter()
            .map(|r| ProjectMember {
                user_id: r.get("user_id"),
                invited_by: r.get("invited_by"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn remove_member(
        &self,
        project_id: uuid::Uuid,
        user_id: &str,
    ) -> Result<bool, ProjectsError> {
        let result = sqlx::query("DELETE FROM project_member WHERE project_id = ? AND user_id = ?")
            .bind(project_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(db_error("remove_member"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn list_invitations(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Vec<ProjectInvitation>, ProjectsError> {
        let sql = format!("{INVITATION_COLUMNS} WHERE project_id = ? ORDER BY created_at, email");
        let rows = sqlx::query(&sql)
            .bind(project_id)
            .fetch_all(&self.pool)
            .await
            .map_err(db_error("list_invitations"))?;
        Ok(rows.iter().map(invitation).collect())
    }

    async fn get_invitation(
        &self,
        invitation_id: uuid::Uuid,
    ) -> Result<Option<ProjectInvitation>, ProjectsError> {
        let sql = format!("{INVITATION_COLUMNS} WHERE id = ?");
        let row = sqlx::query(&sql)
            .bind(invitation_id)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_error("get_invitation"))?;
        Ok(row.as_ref().map(invitation))
    }

    async fn create_invitation(
        &self,
        project_id: uuid::Uuid,
        email: &str,
        user_id: Option<&str>,
        invited_by: &str,
    ) -> Result<(ProjectInvitation, bool), ProjectsError> {
        let inserted = sqlx::query(
            "INSERT INTO project_invitation (id, project_id, email, user_id, invited_by)
                VALUES (?, ?, ?, ?, ?)
                ON CONFLICT(project_id, email) DO NOTHING
                RETURNING id, project_id, email, user_id, invited_by, created_at",
        )
        .bind(uuid::Uuid::new_v4())
        .bind(project_id)
        .bind(email)
        .bind(user_id)
        .bind(invited_by)
        .fetch_optional(&self.pool)
        .await
        .map_err(db_error("create_invitation"))?;
        if let Some(row) = inserted {
            return Ok((invitation(&row), true));
        }
        let sql = format!("{INVITATION_COLUMNS} WHERE project_id = ? AND email = ?");
        let row = sqlx::query(&sql)
            .bind(project_id)
            .bind(email)
            .fetch_one(&self.pool)
            .await
            .map_err(db_error("create_invitation lookup"))?;
        Ok((invitation(&row), false))
    }

    async fn delete_invitation(
        &self,
        project_id: uuid::Uuid,
        invitation_id: uuid::Uuid,
    ) -> Result<bool, ProjectsError> {
        let result = sqlx::query("DELETE FROM project_invitation WHERE id = ? AND project_id = ?")
            .bind(invitation_id)
            .bind(project_id)
            .execute(&self.pool)
            .await
            .map_err(db_error("delete_invitation"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> Result<Vec<uuid::Uuid>, ProjectsError> {
        if emails.is_empty() {
            return Ok(Vec::new());
        }
        let mut tx = self.pool.begin().await.map_err(db_error("claim begin"))?;
        // sqlite can't bind an array; a user has a handful of verified
        // addresses at most, so an expanded placeholder list is fine.
        let placeholders = vec!["?"; emails.len()].join(", ");

        // Someone invited to a project they own (e.g. under a second
        // address) has nothing to accept.
        let own = format!(
            "DELETE FROM project_invitation
                WHERE user_id IS NULL AND email IN ({placeholders})
                  AND project_id IN (SELECT id FROM project WHERE user_id = ?)"
        );
        let mut query = sqlx::query(&own);
        for email in emails {
            query = query.bind(email);
        }
        query
            .bind(user_id)
            .execute(&mut *tx)
            .await
            .map_err(db_error("claim drop own"))?;

        let claim = format!(
            "UPDATE project_invitation SET user_id = ?
                WHERE user_id IS NULL AND email IN ({placeholders})
                RETURNING project_id"
        );
        let mut query = sqlx::query(&claim).bind(user_id);
        for email in emails {
            query = query.bind(email);
        }
        let rows = query
            .fetch_all(&mut *tx)
            .await
            .map_err(db_error("claim update"))?;
        tx.commit().await.map_err(db_error("claim commit"))?;
        Ok(rows
            .iter()
            .map(|r| r.get::<uuid::Uuid, _>("project_id"))
            .collect())
    }

    async fn list_received_invitations(
        &self,
        user_id: &str,
    ) -> Result<Vec<ReceivedInvitation>, ProjectsError> {
        let rows = sqlx::query(
            "SELECT i.id, i.project_id, p.name, h.hostname, i.invited_by, i.created_at
                FROM project_invitation i
                JOIN project p ON p.id = i.project_id
                JOIN host h ON p.host_id = h.id
                WHERE i.user_id = ?
                ORDER BY i.created_at DESC, i.id",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(db_error("list_received_invitations"))?;
        Ok(rows
            .iter()
            .map(|r| ReceivedInvitation {
                id: r.get::<uuid::Uuid, _>("id"),
                project_id: r.get::<uuid::Uuid, _>("project_id"),
                project_name: ProjectName::new(r.get::<String, _>("name")),
                hostname: HostName::new(r.get::<String, _>("hostname")),
                invited_by: r.get("invited_by"),
                created_at: r.get("created_at"),
            })
            .collect())
    }

    async fn accept_invitation(
        &self,
        invitation_id: uuid::Uuid,
        user_id: &str,
    ) -> Result<Option<uuid::Uuid>, ProjectsError> {
        let mut tx = self.pool.begin().await.map_err(db_error("accept begin"))?;
        let Some(row) = sqlx::query(
            "DELETE FROM project_invitation WHERE id = ? AND user_id = ?
                RETURNING project_id, invited_by",
        )
        .bind(invitation_id)
        .bind(user_id)
        .fetch_optional(&mut *tx)
        .await
        .map_err(db_error("accept delete"))?
        else {
            return Ok(None);
        };
        let project_id: uuid::Uuid = row.get("project_id");
        let invited_by: String = row.get("invited_by");
        sqlx::query(
            "INSERT INTO project_member (project_id, user_id, invited_by) VALUES (?, ?, ?)
                ON CONFLICT(project_id, user_id) DO NOTHING",
        )
        .bind(project_id)
        .bind(user_id)
        .bind(&invited_by)
        .execute(&mut *tx)
        .await
        .map_err(db_error("accept insert member"))?;
        tx.commit().await.map_err(db_error("accept commit"))?;
        Ok(Some(project_id))
    }

    async fn decline_invitation(
        &self,
        invitation_id: uuid::Uuid,
        user_id: &str,
    ) -> Result<bool, ProjectsError> {
        let result = sqlx::query("DELETE FROM project_invitation WHERE id = ? AND user_id = ?")
            .bind(invitation_id)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .map_err(db_error("decline_invitation"))?;
        Ok(result.rows_affected() > 0)
    }

    async fn get_deployments(
        &self,
        project_id: uuid::Uuid,
        service: Option<&ServiceName>,
        limit: i64,
    ) -> Result<Vec<Deployment>, ProjectsError> {
        let rows = match service {
            Some(service) => {
                let sql = format!(
                    "{DEPLOYMENT_COLUMNS} WHERE p.id = ? AND s.name = ?
                        ORDER BY d.created_at DESC LIMIT ?"
                );
                sqlx::query(&sql)
                    .bind(project_id)
                    .bind(service.as_str())
                    .bind(limit)
                    .fetch_all(&self.pool)
                    .await
            }
            None => {
                let sql = format!(
                    "{DEPLOYMENT_COLUMNS} WHERE p.id = ? ORDER BY d.created_at DESC LIMIT ?"
                );
                sqlx::query(&sql)
                    .bind(project_id)
                    .bind(limit)
                    .fetch_all(&self.pool)
                    .await
            }
        }
        .map_err(db_error("get_project_deployments"))?;
        Ok(rows.iter().map(deployment).collect())
    }

    async fn get_latest_rollout(
        &self,
        project_id: uuid::Uuid,
    ) -> Result<Option<Deployment>, ProjectsError> {
        let sql = format!(
            "{DEPLOYMENT_COLUMNS} WHERE p.id = ? AND d.status NOT IN (?, ?)
                ORDER BY d.created_at DESC LIMIT 1"
        );
        let row = sqlx::query(&sql)
            .bind(project_id)
            .bind(DeploymentStatus::NoUpdate as u8)
            .bind(DeploymentStatus::TestMessage as u8)
            .fetch_optional(&self.pool)
            .await
            .map_err(db_error("get_latest_rollout"))?;
        Ok(row.as_ref().map(deployment))
    }
}
