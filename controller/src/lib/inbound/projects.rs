//! Project-centric internal endpoints: the project overview, per-project reads
//! for owners and co-maintainers, sharing, and project notifiers.
//!
//! Everything here is keyed by project id rather than (host, project name): a
//! shared project belongs to another account, and a member may have a project
//! of the same name themselves. Every handler first resolves the id to a
//! [`ProjectAccess`] for the calling user (404 when they have none), then reads
//! the owner's container state, metrics, logs and pending updates, which are
//! all stored under the owner's user id.

// Handlers are generic over one type per domain service, like in `server`.
#![allow(clippy::type_complexity)]

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use axum::routing::{get, post};
use axum::{Extension, Router};
use chatterbox::message::Message;
use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use ts_rs::TS;

use crate::domain::alerts::port::AlertsService;
use crate::domain::billing::ports::BillingService;
use crate::domain::container_state::models::state::{ContainerStateData, HostProjectState};
use crate::domain::container_state::port::ContainerStateService;
use crate::domain::deployments::models::deployment::Deployment;
use crate::domain::deployments::ports::DeploymentsService;
use crate::domain::metrics::models::{LatestMetric, RETENTION_DAYS};
use crate::domain::metrics::port::MetricsService;
use crate::domain::notifiers::models::{
    EmailConfig, Notifier, NotifierConfig, NotifierKind, NotifierScope, NotifierSummary,
};
use crate::domain::notifiers::ports::NotifierService;
use crate::domain::projects::models::{
    ProjectAccess, ProjectInvitation, ProjectRole, ProjectsError, ReceivedInvitation,
};
use crate::domain::projects::ports::ProjectsService;
use crate::domain::tokens::ports::TokenService;
use crate::inbound::server::{
    ApiResponse, AppState, ContainerLogsResponse, ContainerStateResponse, ContainerStateResponses,
    MetricPointResponse, ServiceMetricsResponse, SetEnabledRequest, UserId, create_notifier_for,
    inspect_to_sorted_value, test_notifier_in,
};
use crate::outbound::notification_dispatch::{EmailDispatchConfig, dispatch_one_async};
use crate::outbound::pending_updates_memory::PendingUpdate;
use crate::sse::ControllerEvent;

/// Deployments returned by the project and service deployment lists.
const DEPLOYMENTS_LIMIT: i64 = 50;
/// Addresses accepted by one claim call. A user has a handful at most.
const MAX_CLAIM_EMAILS: usize = 20;
/// Longest inviter display name echoed into an invitation email.
const MAX_INVITER_LEN: usize = 200;

// ── Wire types ────────────────────────────────────────────────────────────────

/// One service on a project tile: enough to render health and a name list
/// without shipping the full inspect blob.
#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ProjectServiceSummary {
    pub name: ServiceName,
    pub image: Option<String>,
    /// Docker's `State.Status`: created, running, paused, restarting,
    /// removing, exited or dead.
    pub status: Option<String>,
    /// Docker's `State.Health.Status` (starting, healthy, unhealthy) when the
    /// container defines a health check.
    pub health: Option<String>,
    pub started_at: Option<String>,
    #[ts(type = "number")]
    pub restart_count: i64,
}

/// Latest resource usage summed over the project's current services.
#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ProjectMetricsSummary {
    pub cpu_pct: f64,
    #[ts(type = "number")]
    pub mem_bytes: u64,
    #[ts(type = "number")]
    pub mem_limit_bytes: u64,
    pub recorded_at: DateTime<Utc>,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ProjectSummaryResponse {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    pub name: ProjectName,
    pub hostname: HostName,
    /// The calling user's role, not the project's.
    pub role: ProjectRole,
    pub created_at: String,
    /// When the agent last reported this project; `None` if it never did.
    pub last_updated: Option<DateTime<Utc>>,
    pub services: Vec<ProjectServiceSummary>,
    /// Most recent rollout attempt (success, failure or rollback).
    pub latest_deployment: Option<Deployment>,
    pub metrics: Option<ProjectMetricsSummary>,
    #[ts(type = "number")]
    pub pending_updates: usize,
    #[ts(type = "number")]
    pub member_count: i64,
    #[ts(type = "number")]
    pub notifier_count: i64,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ProjectMemberResponse {
    pub user_id: String,
    pub role: ProjectRole,
    pub invited_by: Option<String>,
    pub created_at: String,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ProjectInvitationResponse {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    pub email: String,
    /// The invitee's account once known; `None` while waiting for them to
    /// sign up.
    pub user_id: Option<String>,
    pub invited_by: String,
    pub created_at: String,
}

impl From<ProjectInvitation> for ProjectInvitationResponse {
    fn from(i: ProjectInvitation) -> Self {
        Self {
            id: i.id,
            email: i.email,
            user_id: i.user_id,
            invited_by: i.invited_by,
            created_at: i.created_at,
        }
    }
}

/// An invitation waiting for the calling user's answer.
#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ReceivedInvitationResponse {
    #[ts(type = "string")]
    pub id: uuid::Uuid,
    #[ts(type = "string")]
    pub project_id: uuid::Uuid,
    pub project_name: ProjectName,
    pub hostname: HostName,
    pub invited_by: String,
    pub created_at: String,
}

impl From<ReceivedInvitation> for ReceivedInvitationResponse {
    fn from(i: ReceivedInvitation) -> Self {
        Self {
            id: i.id,
            project_id: i.project_id,
            project_name: i.project_name,
            hostname: i.hostname,
            invited_by: i.invited_by,
            created_at: i.created_at,
        }
    }
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ProjectDetailResponse {
    pub project: ProjectSummaryResponse,
    /// The owner first, then the co-maintainers.
    pub members: Vec<ProjectMemberResponse>,
    pub invitations: Vec<ProjectInvitationResponse>,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct InviteToProjectResponse {
    pub invitation: ProjectInvitationResponse,
    /// `false` when the address already had a pending invitation, in which
    /// case the caller should not send another invitation email.
    pub created: bool,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct ClaimInvitationsResponse {
    /// Projects whose invitations now wait for the user to accept them.
    #[ts(type = "Array<string>")]
    pub project_ids: Vec<uuid::Uuid>,
}

#[derive(TS, Serialize, Deserialize)]
#[ts(export)]
pub struct AcceptInvitationResponse {
    #[ts(type = "string")]
    pub project_id: uuid::Uuid,
}

#[derive(Deserialize)]
struct InviteBody {
    email: String,
    /// The invitee's account, when the BFF resolved `email` to an existing
    /// user with that verified address. The controller then emails them the
    /// invitation itself; otherwise the BFF sends a sign-up invitation.
    #[serde(default)]
    user_id: Option<String>,
    /// How to name the inviting owner in that email.
    #[serde(default)]
    inviter: Option<String>,
}

#[derive(Deserialize)]
struct ClaimBody {
    emails: Vec<String>,
}

// ── Helpers ──────────────────────────────────────────────────────────────────

fn error_body(status: StatusCode, message: String) -> Response {
    (
        status,
        Json(ApiResponse::<()> {
            success: false,
            data: None,
            error: Some(message),
        }),
    )
        .into_response()
}

fn projects_error(e: ProjectsError) -> Response {
    match e {
        ProjectsError::NotFound => StatusCode::NOT_FOUND.into_response(),
        ProjectsError::Forbidden => error_body(StatusCode::FORBIDDEN, e.to_string()),
        ProjectsError::Invalid(msg) => error_body(StatusCode::BAD_REQUEST, msg),
        ProjectsError::UnknownError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

fn non_empty(s: String) -> Option<String> {
    (!s.is_empty()).then_some(s)
}

fn service_summary(
    name: &ServiceName,
    inspect: &bollard::models::ContainerInspectResponse,
) -> ProjectServiceSummary {
    let state = inspect.state.as_ref();
    ProjectServiceSummary {
        name: name.clone(),
        image: inspect
            .config
            .as_ref()
            .and_then(|c| c.image.clone())
            .or_else(|| inspect.image.clone()),
        status: state
            .and_then(|s| s.status)
            .map(|s| s.to_string())
            .and_then(non_empty),
        health: state
            .and_then(|s| s.health.as_ref())
            .and_then(|h| h.status)
            .map(|s| s.to_string())
            .and_then(non_empty),
        started_at: state.and_then(|s| s.started_at.clone()),
        restart_count: inspect.restart_count.unwrap_or(0),
    }
}

fn summarize_project(
    access: &ProjectAccess,
    state: Option<&HostProjectState>,
    latest_metrics: &[LatestMetric],
    pending: &[PendingUpdate],
    latest_deployment: Option<Deployment>,
) -> ProjectSummaryResponse {
    let mut services: Vec<ProjectServiceSummary> = state
        .map(|s| {
            s.services
                .iter()
                .map(|(name, svc)| service_summary(name, &svc.inspect))
                .collect()
        })
        .unwrap_or_default();
    services.sort_by(|a, b| a.name.0.cmp(&b.name.0));

    // Only count samples of services the project currently runs: the last
    // sample of a removed service lingers for the whole retention window.
    let current: Vec<&LatestMetric> = latest_metrics
        .iter()
        .filter(|m| m.hostname == access.hostname && m.project_name == access.name)
        .filter(|m| services.iter().any(|s| s.name == m.service_name))
        .collect();
    let metrics = current
        .iter()
        .map(|m| m.point.recorded_at)
        .max()
        .map(|recorded_at| ProjectMetricsSummary {
            cpu_pct: current.iter().map(|m| m.point.cpu_pct).sum(),
            mem_bytes: current.iter().map(|m| m.point.mem_bytes).sum(),
            mem_limit_bytes: current.iter().map(|m| m.point.mem_limit_bytes).sum(),
            recorded_at,
        });

    ProjectSummaryResponse {
        id: access.id,
        name: access.name.clone(),
        hostname: access.hostname.clone(),
        role: access.role,
        created_at: access.created_at.clone(),
        last_updated: state.map(|s| s.last_updated),
        services,
        // Tiles don't render log tails; keep the listing light.
        latest_deployment: latest_deployment.map(|d| Deployment { logs: None, ..d }),
        metrics,
        pending_updates: pending
            .iter()
            .filter(|u| u.hostname == access.hostname && u.project_name == access.name)
            .count(),
        member_count: access.member_count,
        notifier_count: access.notifier_count,
    }
}

/// Build overview tiles, reading each owner's state, metrics and pending
/// updates once however many of their projects are listed.
async fn summarize_projects<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    state: &AppState<DS, CS, TS, NS, BS, MS, AS, PS>,
    projects: &[ProjectAccess],
) -> Vec<ProjectSummaryResponse> {
    let mut states: HashMap<&str, ContainerStateData> = HashMap::new();
    let mut metrics: HashMap<&str, Vec<LatestMetric>> = HashMap::new();
    let mut pending: HashMap<&str, Vec<PendingUpdate>> = HashMap::new();
    for p in projects {
        let owner = p.owner_id.as_str();
        if states.contains_key(owner) {
            continue;
        }
        states.insert(
            owner,
            state
                .container_state_service
                .get_container_states(owner)
                .await,
        );
        metrics.insert(owner, state.metrics_service.get_latest_metrics(owner).await);
        pending.insert(owner, state.pending_updates.get_all(owner).await);
    }

    let mut out = Vec::with_capacity(projects.len());
    for p in projects {
        let owner = p.owner_id.as_str();
        let latest = state
            .projects_service
            .get_latest_rollout(p)
            .await
            .unwrap_or_else(|e| {
                error!("latest rollout lookup failed for project {}: {e:?}", p.id);
                None
            });
        let project_state = states
            .get(owner)
            .and_then(|s| s.get(&p.hostname))
            .and_then(|h| h.get(&p.name));
        out.push(summarize_project(
            p,
            project_state,
            metrics.get(owner).map(Vec::as_slice).unwrap_or_default(),
            pending.get(owner).map(Vec::as_slice).unwrap_or_default(),
            latest,
        ));
    }
    out
}

/// Strip control characters and cap the length of a caller-supplied display
/// name before it goes into an email body.
fn sanitize_inviter(raw: &str) -> Option<String> {
    let cleaned: String = raw
        .chars()
        .filter(|c| !c.is_control())
        .take(MAX_INVITER_LEN)
        .collect();
    let cleaned = cleaned.trim().to_string();
    (!cleaned.is_empty()).then_some(cleaned)
}

/// Tell an existing user they were invited to co-maintain a project, through
/// the controller-wide email transport. The link goes to the projects page,
/// where the invitation waits to be accepted or declined. Fire-and-forget;
/// skipped when email delivery isn't configured (the invitation still shows
/// up in the dashboard).
fn send_invitation_email(
    email: Option<EmailDispatchConfig>,
    recipient: String,
    inviter: Option<String>,
    access: &ProjectAccess,
    dashboard_url: &str,
) {
    let Some(email) = email else { return };
    let who = inviter.unwrap_or_else(|| "A Hoister user".to_string());
    let title = format!(
        "{who} invited you to co-maintain {} on Hoister",
        access.name.as_str()
    );
    let body = format!(
        "{who} invited you to co-maintain the project \"{}\" (host {}) on Hoister. Once \
         you accept, you can see its services, deployments and resource usage, deploy \
         pending updates and set up notifications for it.\n\nAccept or decline the \
         invitation: {}/projects",
        access.name.as_str(),
        access.hostname.as_str(),
        dashboard_url.trim_end_matches('/'),
    );
    let notifier = Notifier {
        id: uuid::Uuid::nil(),
        user_id: access.owner_id.clone(),
        project_id: Some(access.id),
        kind: NotifierKind::Email,
        config: NotifierConfig::Email(EmailConfig { recipient }),
        enabled: true,
        created_at: String::new(),
    };
    let project_id = access.id;
    tokio::spawn(async move {
        if let Err(e) = dispatch_one_async(notifier, Message::new(title, body), Some(email)).await {
            error!("invitation email for project {project_id} failed: {e}");
        }
    });
}

// ── Overview ─────────────────────────────────────────────────────────────────

/// Every project the user owns or co-maintains, as overview tiles.
async fn list_projects<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Response {
    let projects = match state.projects_service.list_projects(&user_id).await {
        Ok(p) => p,
        Err(e) => return projects_error(e),
    };
    let summaries = summarize_projects(&state, &projects).await;
    Json(ApiResponse::success(summaries)).into_response()
}

async fn get_project<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    let (members, invitations) = match (
        state.projects_service.list_members(&access).await,
        state.projects_service.list_invitations(&access).await,
    ) {
        (Ok(m), Ok(i)) => (m, i),
        (Err(e), _) | (_, Err(e)) => return projects_error(e),
    };
    let project = summarize_projects(&state, std::slice::from_ref(&access))
        .await
        .pop()
        .expect("one summary per project");

    let owner = ProjectMemberResponse {
        user_id: access.owner_id.clone(),
        role: ProjectRole::Owner,
        invited_by: None,
        created_at: access.created_at.clone(),
    };
    let members = std::iter::once(owner)
        .chain(members.into_iter().map(|m| ProjectMemberResponse {
            user_id: m.user_id,
            role: ProjectRole::Member,
            invited_by: m.invited_by,
            created_at: m.created_at,
        }))
        .collect();
    Json(ApiResponse::success(ProjectDetailResponse {
        project,
        members,
        invitations: invitations.into_iter().map(Into::into).collect(),
    }))
    .into_response()
}

/// Owner only: retire the project. Cascades to everything attached to it,
/// including memberships and project notifiers. The agent recreates it (as a
/// fresh, unshared project) if it is still running and labelled.
async fn delete_project<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state.projects_service.delete_project(&access).await {
        Ok(deleted) => {
            state
                .pending_updates
                .remove_project(&access.owner_id, &access.hostname, &access.name)
                .await;
            if deleted {
                StatusCode::NO_CONTENT.into_response()
            } else {
                StatusCode::NOT_FOUND.into_response()
            }
        }
        Err(e) => projects_error(e),
    }
}

// ── Services, deployments, metrics, logs, updates ────────────────────────────

async fn list_project_services<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    let mut all = state
        .container_state_service
        .get_container_states(&access.owner_id)
        .await;
    let mut only_this: ContainerStateData = HashMap::new();
    if let Some(project_state) = all
        .get_mut(&access.hostname)
        .and_then(|projects| projects.remove(&access.name))
    {
        only_this
            .entry(access.hostname.clone())
            .or_default()
            .insert(access.name.clone(), project_state);
    }
    Json(ContainerStateResponses::from(only_this)).into_response()
}

async fn get_project_service<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, service_name)): Path<(uuid::Uuid, ServiceName)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    let Some(host_project_state) = state
        .container_state_service
        .get_container_state(
            &access.owner_id,
            &access.hostname,
            &access.name,
            &service_name,
        )
        .await
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let Some(service_state) = host_project_state.services.into_values().next() else {
        return StatusCode::NOT_FOUND.into_response();
    };
    Json(ApiResponse::success(ContainerStateResponse {
        hostname: access.hostname,
        project_name: access.name,
        service_name,
        container_inspections: inspect_to_sorted_value(&service_state.inspect),
        last_logs: service_state.last_logs,
        last_updated: host_project_state.last_updated,
    }))
    .into_response()
}

async fn get_project_service_metrics<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, service_name)): Path<(uuid::Uuid, ServiceName)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    let since = Utc::now() - chrono::Duration::days(RETENTION_DAYS);
    let points = state
        .metrics_service
        .get_service_metrics(
            &access.owner_id,
            &access.hostname,
            &access.name,
            &service_name,
            since,
        )
        .await
        .into_iter()
        .map(MetricPointResponse::from)
        .collect();
    Json(ApiResponse::success(ServiceMetricsResponse {
        hostname: access.hostname,
        project_name: access.name,
        service_name,
        points,
    }))
    .into_response()
}

async fn get_project_deployments<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .projects_service
        .get_deployments(&access, None, DEPLOYMENTS_LIMIT)
        .await
    {
        Ok(d) => Json(ApiResponse::success(d)).into_response(),
        Err(e) => projects_error(e),
    }
}

async fn get_project_service_deployments<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, service_name)): Path<(uuid::Uuid, ServiceName)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .projects_service
        .get_deployments(&access, Some(&service_name), DEPLOYMENTS_LIMIT)
        .await
    {
        Ok(d) => Json(ApiResponse::success(d)).into_response(),
        Err(e) => projects_error(e),
    }
}

async fn request_project_service_logs<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, service_name)): Path<(uuid::Uuid, ServiceName)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    // Agents listen on the owner's event stream.
    let event = ControllerEvent::RequestLogs((access.hostname, access.name, service_name));
    let _ = state.event_tx.send((access.owner_id, event));
    StatusCode::ACCEPTED.into_response()
}

async fn get_project_service_logs<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, service_name)): Path<(uuid::Uuid, ServiceName)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .logs
        .get(
            &access.owner_id,
            &access.hostname,
            &access.name,
            &service_name,
        )
        .await
    {
        Some(entry) => Json(ApiResponse::success(ContainerLogsResponse {
            logs: entry.logs,
            received_at: entry.received_at,
        }))
        .into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_project_pending_updates<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    let updates: Vec<PendingUpdate> = state
        .pending_updates
        .get_all(&access.owner_id)
        .await
        .into_iter()
        .filter(|u| u.hostname == access.hostname && u.project_name == access.name)
        .collect();
    Json(updates).into_response()
}

/// Owners and members alike may roll out a pending update — deploying is the
/// point of co-maintaining a project.
async fn apply_project_update<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, service_name)): Path<(uuid::Uuid, ServiceName)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    state
        .pending_updates
        .remove(
            &access.owner_id,
            &access.hostname,
            &access.name,
            &service_name,
        )
        .await;
    let event = ControllerEvent::ApplyUpdate((access.hostname, access.name, service_name));
    let _ = state.event_tx.send((access.owner_id, event));
    StatusCode::OK.into_response()
}

// ── Sharing ──────────────────────────────────────────────────────────────────
// Nobody gains access to a project without agreeing to it: the owner invites
// an email address, and the membership only exists once the invitee accepts.

/// The owner may remove anyone; a member may remove themselves (leave).
async fn remove_project_member<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, member_id)): Path<(uuid::Uuid, String)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .projects_service
        .remove_member(&access, &user_id, &member_id)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => projects_error(e),
    }
}

/// Owner only: invite an email address to co-maintain the project.
///
/// With `user_id` (the BFF resolved the address to an existing account) the
/// invitation is addressed to that user right away and the controller emails
/// them. Without it, sending the sign-up invitation is the BFF's job (it talks
/// to the identity provider) and it should only do so when `created` is true;
/// the invitation gets addressed to the new account when they sign up.
async fn invite_to_project<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
    Json(body): Json<InviteBody>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    let invitee = body.user_id.filter(|u| !u.is_empty());
    if let Some(invitee) = &invitee
        && access.is_owner()
    {
        // The invitee may never have called the controller yet, and
        // invitations reference the users table.
        state.billing_service.upsert_user(invitee).await;
    }
    let (invitation, created) = match state
        .projects_service
        .invite(&access, &body.email, invitee.as_deref())
        .await
    {
        Ok(result) => result,
        Err(e) => return projects_error(e),
    };
    if created && invitee.is_some() {
        send_invitation_email(
            state.email.clone(),
            invitation.email.clone(),
            body.inviter.as_deref().and_then(sanitize_inviter),
            &access,
            &state.dashboard_url,
        );
    }
    Json(ApiResponse::success(InviteToProjectResponse {
        invitation: invitation.into(),
        created,
    }))
    .into_response()
}

async fn revoke_project_invitation<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, invitation_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .projects_service
        .revoke_invitation(&access, invitation_id)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => projects_error(e),
    }
}

/// Address pending invitations for the caller's email addresses to their
/// account, so they show up among the invitations they can accept. The BFF
/// must only pass addresses the identity provider has verified for this user
/// — the controller cannot check that itself.
async fn claim_invitations<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(body): Json<ClaimBody>,
) -> Response {
    if body.emails.len() > MAX_CLAIM_EMAILS {
        return error_body(StatusCode::BAD_REQUEST, "Too many addresses".to_string());
    }
    match state
        .projects_service
        .claim_invitations(&user_id, &body.emails)
        .await
    {
        Ok(project_ids) => Json(ApiResponse::success(ClaimInvitationsResponse {
            project_ids,
        }))
        .into_response(),
        Err(e) => projects_error(e),
    }
}

/// Invitations waiting for the caller to accept or decline them.
async fn list_received_invitations<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Response {
    match state
        .projects_service
        .list_received_invitations(&user_id)
        .await
    {
        Ok(invitations) => {
            let invitations: Vec<ReceivedInvitationResponse> =
                invitations.into_iter().map(Into::into).collect();
            Json(ApiResponse::success(invitations)).into_response()
        }
        Err(e) => projects_error(e),
    }
}

/// Accept an invitation addressed to the caller, becoming a co-maintainer.
/// Anyone else's invitation is a 404.
async fn accept_invitation<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(invitation_id): Path<uuid::Uuid>,
) -> Response {
    match state
        .projects_service
        .accept_invitation(&user_id, invitation_id)
        .await
    {
        Ok(project_id) => Json(ApiResponse::success(AcceptInvitationResponse {
            project_id,
        }))
        .into_response(),
        Err(e) => projects_error(e),
    }
}

async fn decline_invitation<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(invitation_id): Path<uuid::Uuid>,
) -> Response {
    match state
        .projects_service
        .decline_invitation(&user_id, invitation_id)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => projects_error(e),
    }
}

// ── Project notifiers ────────────────────────────────────────────────────────
// Shared by everyone with access to the project. They belong to the owner's
// account (so the owner's plan decides which kinds are allowed) and receive
// only this project's events, in addition to the owner's account-wide ones.

async fn list_project_notifiers<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .notifier_service
        .list_notifiers(NotifierScope::Project(access.id))
        .await
    {
        Ok(notifiers) => {
            let summaries: Vec<NotifierSummary> = notifiers.iter().map(Into::into).collect();
            Json(ApiResponse::success(summaries)).into_response()
        }
        Err(e) => {
            error!("Error listing notifiers of project {project_id}: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn create_project_notifier<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(project_id): Path<uuid::Uuid>,
    Json(config): Json<NotifierConfig>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    create_notifier_for(
        state.notifier_service.as_ref(),
        state.billing_service.as_ref(),
        &access.owner_id,
        Some(access.id),
        config,
    )
    .await
}

async fn delete_project_notifier<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, notifier_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .notifier_service
        .delete_notifier(NotifierScope::Project(access.id), notifier_id)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!("Error deleting notifier {notifier_id} of project {project_id}: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn set_project_notifier_enabled<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, notifier_id)): Path<(uuid::Uuid, uuid::Uuid)>,
    Json(req): Json<SetEnabledRequest>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    match state
        .notifier_service
        .set_enabled(NotifierScope::Project(access.id), notifier_id, req.enabled)
        .await
    {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => StatusCode::NOT_FOUND.into_response(),
        Err(e) => {
            error!("Error toggling notifier {notifier_id} of project {project_id}: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn test_project_notifier<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS, AS, PS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_id, notifier_id)): Path<(uuid::Uuid, uuid::Uuid)>,
) -> Response {
    let access = match state
        .projects_service
        .get_project(&user_id, project_id)
        .await
    {
        Ok(a) => a,
        Err(e) => return projects_error(e),
    };
    test_notifier_in(
        state.notifier_service.as_ref(),
        state.billing_service.as_ref(),
        NotifierScope::Project(access.id),
        notifier_id,
        state.email.clone(),
    )
    .await
}

/// Routes merged into the internal (BFF-facing) router, behind its
/// `X-User-Id` middleware.
pub(crate) fn routes<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
    AS: AlertsService,
    PS: ProjectsService,
>() -> Router<AppState<DS, CS, TS, NS, BS, MS, AS, PS>> {
    Router::new()
        .route(
            "/projects",
            get(list_projects::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}",
            get(get_project::<DS, CS, TS, NS, BS, MS, AS, PS>)
                .delete(delete_project::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services",
            get(list_project_services::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services/{service}",
            get(get_project_service::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services/{service}/metrics",
            get(get_project_service_metrics::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services/{service}/deployments",
            get(get_project_service_deployments::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services/{service}/logs",
            get(get_project_service_logs::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services/{service}/logs/request",
            post(request_project_service_logs::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/services/{service}/apply",
            post(apply_project_update::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/deployments",
            get(get_project_deployments::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/pending-updates",
            get(get_project_pending_updates::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/members/{user_id}",
            axum::routing::delete(remove_project_member::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/invitations",
            post(invite_to_project::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/invitations/{invitation_id}",
            axum::routing::delete(revoke_project_invitation::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/invitations/claim",
            post(claim_invitations::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/invitations",
            get(list_received_invitations::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/invitations/{invitation_id}/accept",
            post(accept_invitation::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/invitations/{invitation_id}/decline",
            post(decline_invitation::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/notifiers",
            get(list_project_notifiers::<DS, CS, TS, NS, BS, MS, AS, PS>)
                .post(create_project_notifier::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/notifiers/{notifier_id}",
            axum::routing::delete(delete_project_notifier::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/notifiers/{notifier_id}/enabled",
            axum::routing::patch(set_project_notifier_enabled::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
        .route(
            "/projects/{id}/notifiers/{notifier_id}/test",
            post(test_project_notifier::<DS, CS, TS, NS, BS, MS, AS, PS>),
        )
}

#[cfg(test)]
mod tests {
    use super::sanitize_inviter;

    #[test]
    fn sanitize_inviter_strips_control_characters_and_caps_length() {
        assert_eq!(
            sanitize_inviter("Alice\r\nBcc: eve@example.com").as_deref(),
            Some("AliceBcc: eve@example.com")
        );
        assert_eq!(sanitize_inviter("   ").as_deref(), None);
        assert_eq!(
            sanitize_inviter(&"x".repeat(500)).map(|s| s.len()),
            Some(200)
        );
    }
}
