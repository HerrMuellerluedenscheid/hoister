use crate::domain::deployments::models::deployment::Deployment;
use crate::domain::projects::models::{
    MAX_MEMBERS_PER_PROJECT, MAX_PENDING_INVITATIONS_PER_PROJECT, ProjectAccess, ProjectInvitation,
    ProjectMember, ProjectsError, ReceivedInvitation, normalize_email,
};
use crate::domain::projects::ports::{ProjectsRepository, ProjectsService};
use hoister_shared::ServiceName;

/// Most recent deployments a project page may ask for in one go.
const MAX_DEPLOYMENTS: i64 = 200;

#[derive(Clone)]
pub struct Service<PR: ProjectsRepository> {
    repository: PR,
}

impl<PR: ProjectsRepository> Service<PR> {
    pub fn new(repository: PR) -> Self {
        Self { repository }
    }
}

fn require_owner(access: &ProjectAccess) -> Result<(), ProjectsError> {
    if access.is_owner() {
        Ok(())
    } else {
        Err(ProjectsError::Forbidden)
    }
}

impl<PR: ProjectsRepository> ProjectsService for Service<PR> {
    async fn list_projects(&self, user_id: &str) -> Result<Vec<ProjectAccess>, ProjectsError> {
        self.repository.list_accessible_projects(user_id).await
    }

    async fn get_project(
        &self,
        user_id: &str,
        project_id: uuid::Uuid,
    ) -> Result<ProjectAccess, ProjectsError> {
        self.repository
            .get_project_access(user_id, project_id)
            .await?
            .ok_or(ProjectsError::NotFound)
    }

    async fn delete_project(&self, access: &ProjectAccess) -> Result<bool, ProjectsError> {
        require_owner(access)?;
        self.repository.delete_project(access.id).await
    }

    async fn list_members(
        &self,
        access: &ProjectAccess,
    ) -> Result<Vec<ProjectMember>, ProjectsError> {
        self.repository.list_members(access.id).await
    }

    async fn remove_member(
        &self,
        access: &ProjectAccess,
        acting_user: &str,
        member_id: &str,
    ) -> Result<bool, ProjectsError> {
        if !access.is_owner() && acting_user != member_id {
            return Err(ProjectsError::Forbidden);
        }
        self.repository.remove_member(access.id, member_id).await
    }

    async fn list_invitations(
        &self,
        access: &ProjectAccess,
    ) -> Result<Vec<ProjectInvitation>, ProjectsError> {
        self.repository.list_invitations(access.id).await
    }

    async fn invite(
        &self,
        access: &ProjectAccess,
        email: &str,
        user_id: Option<&str>,
    ) -> Result<(ProjectInvitation, bool), ProjectsError> {
        require_owner(access)?;
        let email = normalize_email(email)
            .ok_or_else(|| ProjectsError::Invalid("Invalid email address".to_string()))?;
        let user_id = user_id.filter(|u| !u.is_empty());
        if let Some(user_id) = user_id {
            if user_id == access.owner_id {
                return Err(ProjectsError::Invalid(
                    "That is the owner's address — they already have access.".to_string(),
                ));
            }
            let members = self.repository.list_members(access.id).await?;
            if members.iter().any(|m| m.user_id == user_id) {
                return Err(ProjectsError::Invalid(
                    "This person already has access to the project.".to_string(),
                ));
            }
        }

        let pending = self.repository.list_invitations(access.id).await?;
        if let Some(existing) = pending.iter().find(|i| i.email == email) {
            // Invited before they had an account: address it to them now, so
            // it shows up for them without waiting for the next login claim.
            if existing.user_id.is_none()
                && let Some(user_id) = user_id
            {
                self.repository
                    .claim_invitations(user_id, std::slice::from_ref(&email))
                    .await?;
                let addressed = ProjectInvitation {
                    user_id: Some(user_id.to_string()),
                    ..existing.clone()
                };
                return Ok((addressed, false));
            }
            return Ok((existing.clone(), false));
        }
        if pending.len() >= MAX_PENDING_INVITATIONS_PER_PROJECT {
            return Err(ProjectsError::Invalid(format!(
                "A project can have at most {MAX_PENDING_INVITATIONS_PER_PROJECT} pending \
                 invitations. Revoke some before inviting more people."
            )));
        }
        self.repository
            .create_invitation(access.id, &email, user_id, &access.owner_id)
            .await
    }

    async fn revoke_invitation(
        &self,
        access: &ProjectAccess,
        invitation_id: uuid::Uuid,
    ) -> Result<bool, ProjectsError> {
        require_owner(access)?;
        self.repository
            .delete_invitation(access.id, invitation_id)
            .await
    }

    async fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> Result<Vec<uuid::Uuid>, ProjectsError> {
        let emails: Vec<String> = emails.iter().filter_map(|e| normalize_email(e)).collect();
        if emails.is_empty() {
            return Ok(Vec::new());
        }
        self.repository.claim_invitations(user_id, &emails).await
    }

    async fn list_received_invitations(
        &self,
        user_id: &str,
    ) -> Result<Vec<ReceivedInvitation>, ProjectsError> {
        self.repository.list_received_invitations(user_id).await
    }

    async fn accept_invitation(
        &self,
        user_id: &str,
        invitation_id: uuid::Uuid,
    ) -> Result<uuid::Uuid, ProjectsError> {
        let invitation = self
            .repository
            .get_invitation(invitation_id)
            .await?
            .filter(|i| i.user_id.as_deref() == Some(user_id))
            .ok_or(ProjectsError::NotFound)?;
        let members = self.repository.list_members(invitation.project_id).await?;
        if !members.iter().any(|m| m.user_id == user_id) && members.len() >= MAX_MEMBERS_PER_PROJECT
        {
            return Err(ProjectsError::Invalid(format!(
                "This project already has the maximum of {MAX_MEMBERS_PER_PROJECT} co-maintainers."
            )));
        }
        self.repository
            .accept_invitation(invitation_id, user_id)
            .await?
            .ok_or(ProjectsError::NotFound)
    }

    async fn decline_invitation(
        &self,
        user_id: &str,
        invitation_id: uuid::Uuid,
    ) -> Result<bool, ProjectsError> {
        self.repository
            .decline_invitation(invitation_id, user_id)
            .await
    }

    async fn get_deployments(
        &self,
        access: &ProjectAccess,
        service: Option<&ServiceName>,
        limit: i64,
    ) -> Result<Vec<Deployment>, ProjectsError> {
        self.repository
            .get_deployments(access.id, service, limit.clamp(1, MAX_DEPLOYMENTS))
            .await
    }

    async fn get_latest_rollout(
        &self,
        access: &ProjectAccess,
    ) -> Result<Option<Deployment>, ProjectsError> {
        self.repository.get_latest_rollout(access.id).await
    }
}
