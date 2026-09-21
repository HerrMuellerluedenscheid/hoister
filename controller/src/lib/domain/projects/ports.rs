use crate::domain::deployments::models::deployment::Deployment;
use crate::domain::projects::models::{
    AccessibleDeployment, ProjectAccess, ProjectInvitation, ProjectMember, ProjectsError,
    ReceivedInvitation,
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

    fn remove_member(
        &self,
        project_id: uuid::Uuid,
        user_id: &str,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    fn list_invitations(
        &self,
        project_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<ProjectInvitation>, ProjectsError>> + Send;

    fn get_invitation(
        &self,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<ProjectInvitation>, ProjectsError>> + Send;

    /// Create the pending invitation for `email`, which must already be
    /// normalised, addressed to `user_id` when the invitee already has an
    /// account. When one is already pending it is returned unchanged with
    /// `false`, so repeated invites don't re-send anything.
    fn create_invitation(
        &self,
        project_id: uuid::Uuid,
        email: &str,
        user_id: Option<&str>,
        invited_by: &str,
    ) -> impl Future<Output = Result<(ProjectInvitation, bool), ProjectsError>> + Send;

    fn delete_invitation(
        &self,
        project_id: uuid::Uuid,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    /// Address every not-yet-addressed invitation for one of `emails`
    /// (normalised) to `user_id`, so it shows up among their received
    /// invitations. Invitations to a project the user owns are deleted
    /// instead. Returns the ids of the projects the user was invited to.
    fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, ProjectsError>> + Send;

    /// Invitations addressed to `user_id`, newest first.
    fn list_received_invitations(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<ReceivedInvitation>, ProjectsError>> + Send;

    /// Turn the invitation into a membership of `user_id`, atomically. Only
    /// succeeds for the invitation's addressee; returns the project id.
    fn accept_invitation(
        &self,
        invitation_id: uuid::Uuid,
        user_id: &str,
    ) -> impl Future<Output = Result<Option<uuid::Uuid>, ProjectsError>> + Send;

    /// Delete the invitation if it is addressed to `user_id`.
    fn decline_invitation(
        &self,
        invitation_id: uuid::Uuid,
        user_id: &str,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

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

    /// Most recent deployments across every project `user_id` owns or is a
    /// member of, newest first.
    fn list_accessible_deployments(
        &self,
        user_id: &str,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AccessibleDeployment>, ProjectsError>> + Send;
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

    /// Owner only. `email` is validated and normalised here; `user_id` is the
    /// invitee's account when they already have one (resolved by the caller
    /// from a verified address). The flag is `false` when the address already
    /// had a pending invitation.
    fn invite(
        &self,
        access: &ProjectAccess,
        email: &str,
        user_id: Option<&str>,
    ) -> impl Future<Output = Result<(ProjectInvitation, bool), ProjectsError>> + Send;

    /// Owner only.
    fn revoke_invitation(
        &self,
        access: &ProjectAccess,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

    /// `emails` must be addresses the identity provider verified for
    /// `user_id`; malformed entries are ignored. Claiming only addresses the
    /// invitations to the user — access still needs them to accept.
    fn claim_invitations(
        &self,
        user_id: &str,
        emails: &[String],
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, ProjectsError>> + Send;

    fn list_received_invitations(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<ReceivedInvitation>, ProjectsError>> + Send;

    /// Accept an invitation addressed to `user_id`; returns the project id.
    /// `NotFound` for anyone else's invitation.
    fn accept_invitation(
        &self,
        user_id: &str,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<uuid::Uuid, ProjectsError>> + Send;

    fn decline_invitation(
        &self,
        user_id: &str,
        invitation_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, ProjectsError>> + Send;

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

    /// Most recent deployments across all projects the user can access.
    fn list_accessible_deployments(
        &self,
        user_id: &str,
        limit: i64,
    ) -> impl Future<Output = Result<Vec<AccessibleDeployment>, ProjectsError>> + Send;
}
