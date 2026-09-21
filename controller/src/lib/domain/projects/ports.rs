use crate::domain::deployments::models::deployment::Deployment;
use crate::domain::projects::models::{
    ProjectAccess, ProjectInvitation, ProjectMember, ProjectsError,
};
use hoister_shared::ServiceName;

pub trait ProjectsRepository: Send + Sync + 'static + Clone {
    /// Every project `user_id` owns or is a member of, ordered by name.
    fn list_accessible_projects(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<ProjectAccess>, ProjectsError>> + Send;

    /// One project, if `user_id` owns it or is a member. `None` otherwise.
    fn get_project_access(
        &self,
        user_id: &str,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<ProjectAccess>, ProjectsError>> + Send;

    /// Delete the project row. Cascades to its services, deployments, state,
    /// metrics, members, invitations and project notifiers.
    fn delete_project(
        &self,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    fn list_members(
        &self,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<ProjectMember>, ProjectsError>> + Send;

    /// Returns `false` when the user already was a member.
    fn add_member(
        &self,
        project_id: uuid::Uuid,
        user_id: &str,
        invited_by: &str,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    fn remove_member(
        &self,
        project_id: uuid::Uuid,
        user_id: &str,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    fn list_invitations(
        &self,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<ProjectInvitation>, ProjectsError>> + Send;

    /// Create the pending invitation for `email`, which must already be
    /// normalised. When one is already pending it is returned unchanged with
    /// `false`, so repeated invites don't re-send anything.
    fn create_invitation(
        &self,
        project_id: uuid::Uuid,
        email: &str,
        invited_by: &str,
    ) -> impl Future<Output = Result<(ProjectInvitation, bool), ProjectsError>> + Send;

    fn delete_invitation(
        &self,
        project_id: uuid::Uuid,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    /// Drop any pending invitation for `email` on the project, e.g. once the
    /// person was added directly.
    fn delete_invitation_by_email(
        &self,
        project_id: uuid::Uuid,
        email: &str,
    ) -> impl Future<Output = Result<(), ProjectsError>> + Send;

    /// Turn every pending invitation addressed to one of `emails` (normalised)
    /// into a membership of `user_id` and delete it. Invitations to a project
    /// the user already owns are just deleted. Returns the ids of the projects
    /// the user was added to.
    fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, ProjectsError>> + Send;

    /// Most recent deployments of the project, optionally of one service only.
    fn get_deployments(
        &self,
        project_id: uuid::Uuid,
        service: Option<&ServiceName>,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<Deployment>, ProjectsError>> + Send;

    /// The project's most recent actual rollout attempt, skipping the
    /// "checked, nothing new" (`NoUpdate`) and test-message rows.
    fn get_latest_rollout(
        &self,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<Deployment>, ProjectsError>> + Send;
}

pub trait ProjectsService: Send + Sync + 'static + Clone {
    fn list_projects(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<ProjectAccess>, ProjectsError>> + Send;

    /// Resolve a project for `user_id`, failing with `NotFound` when it
    /// doesn't exist or the user may not see it.
    fn get_project(
        &self,
        user_id: &str,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<ProjectAccess, ProjectsError>> + Send;

    /// Owner only.
    fn delete_project(
        &self,
        access: &ProjectAccess,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    fn list_members(
        &self,
        access: &ProjectAccess,
    ) -> impl Future<Output = Result<Vec<ProjectMember>, ProjectsError>> + Send;

    /// Owner only. Adding the owner themselves is rejected.
    fn add_member(
        &self,
        access: &ProjectAccess,
        user_id: &str,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    /// The owner may remove anyone; a member may only remove themselves
    /// (leave the project).
    fn remove_member(
        &self,
        access: &ProjectAccess,
        acting_user: &str,
        member_id: &str,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    fn list_invitations(
        &self,
        access: &ProjectAccess,
    ) -> impl Future<Output = Result<Vec<ProjectInvitation>, ProjectsError>> + Send;

    /// Owner only. `email` is validated and normalised here. The flag is
    /// `false` when the address already had a pending invitation.
    fn invite(
        &self,
        access: &ProjectAccess,
        email: &str,
    ) -> impl Future<Output = Result<(ProjectInvitation, bool), ProjectsError>> + Send;

    /// Owner only.
    fn revoke_invitation(
        &self,
        access: &ProjectAccess,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    /// Owner only: drop the pending invitation for `email`, if any.
    fn revoke_invitation_by_email(
        &self,
        access: &ProjectAccess,
        email: &str,
    ) -> impl Future<Output = Result<(), ProjectsError>> + Send;

    /// `emails` must be addresses the identity provider verified for
    /// `user_id`; malformed entries are ignored.
    fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, ProjectsError>> + Send;

    fn get_deployments(
        &self,
        access: &ProjectAccess,
        service: Option<&ServiceName>,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<Deployment>, ProjectsError>> + Send;

    fn get_latest_rollout(
        &self,
        access: &ProjectAccess,
    ) -> impl Future<Output = Result<Option<Deployment>, ProjectsError>> + Send;
}
