use axum::extract::DefaultBodyLimit;
use axum::response::IntoResponse;
use axum::{
    Extension, Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::domain::billing::models::{Plan, PlanStatus, Usage};
use crate::domain::billing::ports::BillingService;
use crate::domain::container_state::models::state::{AddContainerStateRequest, ContainerStateData};
use crate::domain::container_state::port::ContainerStateService;
use crate::domain::deployments::models::deployment::{
    CreateDeploymentRequest, Deployment, GetDeploymentError,
};
use crate::domain::deployments::ports::DeploymentsService;
use crate::domain::metrics::models::{AddMetricsRequest, RETENTION_DAYS};
use crate::domain::metrics::port::MetricsService;
use crate::domain::notifiers::models::{NotifierConfig, NotifierSummary};
use crate::domain::notifiers::ports::NotifierService;
use crate::domain::tokens::models::ApiToken;
use crate::domain::tokens::ports::TokenService;
use crate::inbound::audit_log::audit_log_middleware;
use crate::inbound::notifier_validation::validate_config as validate_notifier_config;
use crate::inbound::rate_limit::{RateLimiter, rate_limit_middleware};
use crate::outbound::notification_dispatch::{
    EmailDispatchConfig, dispatch_one_async, dispatch_to_all,
};
use crate::outbound::pending_updates_memory::{PendingUpdate, PendingUpdatesMemory};
use crate::sse::{ControllerEvent, UserScopedEvent, sse_handler};

/// Cap on the JSON payload an agent can POST to the controller. Inspect
/// payloads with many containers and 16 KB log tails per container add up,
/// so we leave generous headroom; this exists to shed abuse, not to enforce
/// product limits.
const AGENT_BODY_LIMIT: usize = 1024 * 1024;
use chatterbox::message::Message;
use hoister_shared::wire::PostContainerMetricsRequest;
use hoister_shared::{
    CreateDeployment, DeploymentStatus, HostName, ProjectName, ServiceName,
    deployment_email_subject,
};
use tokio::sync::broadcast;
use ts_rs::TS;

/// user ID extracted from a verified request, injected as a request extension.
/// For internal (BFF) requests this comes from the `X-User-Id` header (trusted because
/// the internal router is VPC-isolated, not publicly reachable).
/// For agent requests this comes from the per-user token DB lookup.
#[derive(Debug, Clone)]
pub struct UserId(pub String);

/// Shared secret expected from the BFF as `X-Internal-Auth`. When `None`,
/// the internal router only enforces `X-User-Id` — acceptable when the
/// listener is on loopback, but a footgun on a docker bridge. The
/// `create_internal_router` entry point logs a warning in that case.
#[derive(Clone)]
pub struct InternalSecret(pub Option<String>);

#[derive(Clone)]
pub struct AppState<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
> {
    pub deployments_service: Arc<DS>,
    pub container_state_service: Arc<CS>,
    pub token_service: Arc<TS>,
    pub notifier_service: Arc<NS>,
    pub billing_service: Arc<BS>,
    pub metrics_service: Arc<MS>,
    #[cfg(feature = "self-hosted")]
    pub api_secret: Option<String>,
    pub event_tx: broadcast::Sender<UserScopedEvent>,
    pub pending_updates: PendingUpdatesMemory,
    /// Controller-wide email (Resend) delivery settings, or `None` when not
    /// configured. Email notifiers can't dispatch without this.
    pub email: Option<EmailDispatchConfig>,
    /// Public base URL of the dashboard frontend. Notifications append a deep
    /// link to the relevant container details page using this origin.
    pub dashboard_url: String,
}

#[derive(Serialize, Deserialize, Debug, TS)]
#[ts(export)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
}

/// Authentication middleware for the agent-facing router (public, TLS).
///
/// **Cloud build** (no `self-hosted` feature):
///   - `hst_<token>` — per-user agent token, looked up in DB.
///
/// **Self-hosted build** (`self-hosted` feature):
///   - Static API secret — accepted from agents.
///   - `hst_<token>` — per-user agent token.
///   - If no secret is configured at all, auth is skipped (open dev mode).
async fn agent_auth_middleware<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Self-hosted: if no secret is configured, skip auth entirely (open dev mode).
    // Insert a synthetic "local" UserId so downstream handlers can rely on
    // tenant scoping being non-optional.
    #[cfg(feature = "self-hosted")]
    if state.api_secret.is_none() {
        state.billing_service.upsert_user("local").await;
        request.extensions_mut().insert(UserId("local".to_string()));
        return Ok(next.run(request).await);
    }

    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    let bearer = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(str::to_owned);

    let Some(token) = bearer else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    // 1. Static API secret (self-hosted only). We DELIBERATELY ignore any
    // X-User-Id header here: the agent router is reachable over the public
    // network, so honouring caller-supplied tenant identifiers would let
    // anyone with the secret impersonate any user. Only the internal router
    // (VPC-private) trusts X-User-Id.
    #[cfg(feature = "self-hosted")]
    if let Some(ref secret) = state.api_secret
        && token == *secret
    {
        state.billing_service.upsert_user("local").await;
        request.extensions_mut().insert(UserId("local".to_string()));
        return Ok(next.run(request).await);
    }

    // 2. Per-user agent token (hst_ prefix) — look up user ID from DB.
    if token.starts_with("hst_") {
        match state.token_service.find_user_by_token(&token).await {
            Some(user_id) => {
                state.billing_service.upsert_user(&user_id).await;
                request.extensions_mut().insert(UserId(user_id));
                return Ok(next.run(request).await);
            }
            None => return Err(StatusCode::UNAUTHORIZED),
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}

/// Middleware for the internal (BFF-facing) router.
///
/// This router is VPC-isolated — not publicly reachable — so no cryptographic
/// auth is required. We simply trust the `X-User-Id` header set by the BFF
/// (which has already authenticated the user via Clerk).
async fn internal_user_middleware<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(InternalSecret(expected)): Extension<InternalSecret>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    // If a shared secret is configured, every request must carry a matching
    // X-Internal-Auth header. We compare with subtle::ConstantTimeEq to
    // avoid leaking the secret via response-time differences across an
    // attacker on the same docker bridge.
    if let Some(expected) = expected.as_deref() {
        let got = request
            .headers()
            .get("X-Internal-Auth")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");
        if !constant_time_eq(got.as_bytes(), expected.as_bytes()) {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    let Some(user_id) = request
        .headers()
        .get("X-User-Id")
        .and_then(|h| h.to_str().ok())
        .map(str::to_owned)
    else {
        // The BFF must always set X-User-Id. Refuse the request rather than
        // letting it through unscoped.
        return Err(StatusCode::UNAUTHORIZED);
    };
    state.billing_service.upsert_user(&user_id).await;
    request.extensions_mut().insert(UserId(user_id));

    Ok(next.run(request).await)
}

/// Constant-time byte equality. Returns false for differing lengths.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

// Web handlers

async fn health() -> &'static str {
    "OK"
}

/// 402 Payment Required + JSON body. Used uniformly so the BFF can detect
/// paywall failures without parsing free-text error messages.
#[derive(Serialize)]
struct UpgradeBody {
    success: bool,
    error: String,
    required_plan: &'static str,
}

fn upgrade_required(message: &str, required_plan: Plan) -> Response {
    let body = UpgradeBody {
        success: false,
        error: message.to_string(),
        required_plan: required_plan.as_str(),
    };
    (StatusCode::PAYMENT_REQUIRED, Json(body)).into_response()
}

async fn get_me<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Result<Json<ApiResponse<PlanStatus>>, StatusCode> {
    let plan = match state.billing_service.get_plan(&user_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch plan for {user_id}: {e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let states = state
        .container_state_service
        .get_container_states(&user_id)
        .await;
    let projects: std::collections::HashSet<&ProjectName> =
        states.values().flat_map(|projs| projs.keys()).collect();

    let mut notifiers_by_kind: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    if let Ok(notifiers) = state.notifier_service.list_notifiers(&user_id).await {
        for n in &notifiers {
            *notifiers_by_kind
                .entry(n.kind.as_str().to_string())
                .or_insert(0) += 1;
        }
    }

    Ok(Json(ApiResponse::success(PlanStatus {
        plan,
        limits: plan.limits(),
        usage: Usage {
            projects: projects.len() as i64,
            notifiers_by_kind,
        },
    })))
}

#[derive(Deserialize)]
struct SetPlanRequest {
    plan: String,
}

/// Set a user's billing plan. Internal-only: called by the BFF's Stripe
/// webhook handler after it has verified the Stripe signature. The target
/// user comes from the `X-User-Id` header (the BFF sets it to the user the
/// Stripe event belongs to), never from the request body — so a bug in the
/// body can't move the wrong account between tiers.
async fn set_plan<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(req): Json<SetPlanRequest>,
) -> Result<StatusCode, StatusCode> {
    let Some(plan) = Plan::parse(&req.plan) else {
        return Err(StatusCode::BAD_REQUEST);
    };
    match state.billing_service.set_plan(&user_id, plan).await {
        Ok(()) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            error!("set_plan failed for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_tokens<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Result<Json<ApiResponse<Vec<ApiToken>>>, StatusCode> {
    match state.token_service.list_tokens(&user_id).await {
        Ok(tokens) => Ok(Json(ApiResponse::success(tokens))),
        Err(e) => {
            error!("Error listing tokens for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize, Default)]
struct CreateTokenRequest {
    #[serde(default)]
    comment: Option<String>,
}

async fn create_token<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    body: Option<Json<CreateTokenRequest>>,
) -> Result<Json<ApiResponse<ApiToken>>, StatusCode> {
    // Empty bodies and missing Content-Type are both accepted: a plain
    // `POST /tokens` mints an unlabelled token.
    let comment = body
        .map(|Json(b)| b.comment)
        .unwrap_or_default()
        .filter(|s| !s.trim().is_empty());
    match state.token_service.create_token(&user_id, comment).await {
        Ok(token) => Ok(Json(ApiResponse::success(token))),
        Err(e) => {
            error!("Error creating token for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_token<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(token_id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    debug!("delete_token user={user_id} token_id={token_id}");
    match state.token_service.delete_token(&user_id, token_id).await {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Error deleting token {token_id} for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn list_notifiers<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Result<Json<ApiResponse<Vec<NotifierSummary>>>, StatusCode> {
    match state.notifier_service.list_notifiers(&user_id).await {
        Ok(notifiers) => {
            let summaries: Vec<NotifierSummary> = notifiers.iter().map(Into::into).collect();
            Ok(Json(ApiResponse::success(summaries)))
        }
        Err(e) => {
            error!("Error listing notifiers for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_notifier<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(config): Json<NotifierConfig>,
) -> Response {
    let kind = config.kind();
    let plan = match state.billing_service.get_plan(&user_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch plan for {user_id}: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !plan.limits().allows_notifier_kind(kind) {
        return upgrade_required(
            &format!("{kind:?} notifiers require the Pro plan"),
            Plan::Pro,
        );
    }

    // SSRF guard: refuse private/loopback hosts and non-https schemes before
    // we ever persist the config. See `notifier_validation` for the policy.
    if let Err(e) = validate_notifier_config(&config).await {
        let msg = e.user_message();
        error!("Notifier validation rejected for {user_id}: {e:?}");
        return (
            StatusCode::BAD_REQUEST,
            Json(NotifierDispatchError {
                success: false,
                error: msg,
            }),
        )
            .into_response();
    }

    match state
        .notifier_service
        .create_notifier(&user_id, config)
        .await
    {
        Ok(notifier) => {
            let summary: NotifierSummary = (&notifier).into();
            Json(ApiResponse::success(summary)).into_response()
        }
        Err(crate::domain::notifiers::models::NotifierError::InvalidConfig(msg)) => {
            error!("Invalid notifier config from {user_id}: {msg}");
            StatusCode::BAD_REQUEST.into_response()
        }
        Err(e) => {
            error!("Error creating notifier for {user_id}: {e:?}");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

async fn delete_notifier<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(notifier_id): Path<uuid::Uuid>,
) -> Result<StatusCode, StatusCode> {
    match state
        .notifier_service
        .delete_notifier(&user_id, notifier_id)
        .await
    {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Error deleting notifier {notifier_id} for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Deserialize)]
struct SetEnabledRequest {
    enabled: bool,
}

async fn set_notifier_enabled<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(notifier_id): Path<uuid::Uuid>,
    Json(req): Json<SetEnabledRequest>,
) -> Result<StatusCode, StatusCode> {
    match state
        .notifier_service
        .set_enabled(&user_id, notifier_id, req.enabled)
        .await
    {
        Ok(true) => Ok(StatusCode::NO_CONTENT),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            error!("Error toggling notifier {notifier_id} for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[derive(Serialize)]
struct NotifierDispatchError {
    success: bool,
    error: String,
}

/// Dispatch a synthetic test message through a specific notifier. Disabled
/// notifiers are still tested so the user can verify config before flipping
/// the channel on. Unlike deployment-event dispatch, errors are returned
/// (502 + body) so the user knows what's wrong.
async fn test_notifier<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(notifier_id): Path<uuid::Uuid>,
) -> Response {
    let notifiers = match state.notifier_service.list_notifiers(&user_id).await {
        Ok(n) => n,
        Err(e) => {
            error!("test_notifier list failed for {user_id}: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(notifier) = notifiers.into_iter().find(|n| n.id == notifier_id) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let kind = notifier.kind;

    // Refuse to test (and therefore re-validate the credentials of) a
    // notifier whose kind is no longer allowed under the user's current
    // plan. Matches `notify_user`'s dispatch-side filter.
    let plan = match state.billing_service.get_plan(&user_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("test_notifier: plan lookup failed for {user_id}: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if !plan.limits().allows_notifier_kind(kind) {
        return upgrade_required(
            &format!("{kind:?} notifiers require the Pro plan"),
            Plan::Pro,
        );
    }
    let msg = Message::new(
        "Hoister test message".to_string(),
        format!(
            "If you see this, the {kind:?} notifier on your hoister account is configured correctly."
        ),
    );
    match dispatch_one_async(notifier, msg, state.email.clone()).await {
        Ok(()) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => {
            // Full chatterbox error goes to the server log only. We
            // deliberately do NOT echo it back: chatterbox includes the
            // upstream response body in its error string, and the dispatch
            // path can be aimed at internal hosts. Returning the body
            // upstream would turn the Test button into an SSRF readback
            // primitive. The user instead gets a fixed message keyed off
            // the notifier kind and is told to check the channel + logs.
            error!("test_notifier dispatch failed for {user_id}/{notifier_id} ({kind:?}): {e}");
            (
                StatusCode::BAD_GATEWAY,
                Json(NotifierDispatchError {
                    success: false,
                    error: format!(
                        "{kind:?} notifier rejected the test message. Verify the credentials are still valid; if so the operator can see the upstream error in the server log."
                    ),
                }),
            )
                .into_response()
        }
    }
}

async fn get_deployments<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    match state
        .deployments_service
        .get_all_deployments(&user_id)
        .await
    {
        Ok(deployments) => Ok(Json(ApiResponse::success(deployments))),
        Err(e) => {
            error!("Error getting deployments: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_deployments_by_service<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((project_name, service_name)): Path<(ProjectName, ServiceName)>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    debug!("get service by name: {service_name:?}");
    match state
        .deployments_service
        .get_deployments_of_service(&project_name, &service_name, &user_id)
        .await
    {
        Ok(deployments) => Ok(Json(ApiResponse::success(deployments))),
        Err(GetDeploymentError::DeploymentNotFound) => Ok(Json(ApiResponse::success(vec![]))),
        Err(e) => {
            error!("Error getting deployment {service_name:?} | {project_name:?} : {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_deployment<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(payload): Json<CreateDeployment>,
) -> Response {
    // Plan cap also applies here. `post_container_state` blocks a new
    // project from being tracked; `create_deployment` writes into the
    // `project` table independently — without this check a Free user
    // could keep creating fresh project_names via the agent's deployment
    // reports and grow `project`/`service`/`deployment` rows unbounded.
    let plan = match state.billing_service.get_plan(&user_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch plan for {user_id}: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if let Some(max) = plan.limits().max_projects {
        let states = state
            .container_state_service
            .get_container_states(&user_id)
            .await;
        let mut distinct: std::collections::HashSet<ProjectName> = states
            .values()
            .flat_map(|projs| projs.keys().cloned())
            .collect();
        if let Ok(existing) = state
            .deployments_service
            .get_all_deployments(&user_id)
            .await
        {
            for d in &existing {
                distinct.insert(d.project_name.clone());
            }
        }
        if !distinct.contains(&payload.project) {
            distinct.insert(payload.project.clone());
            if distinct.len() as i64 > max {
                return upgrade_required(
                    &format!(
                        "Project '{}' would exceed the Free plan limit of {max} projects. Upgrade to Pro for unlimited projects.",
                        payload.project.as_str(),
                    ),
                    Plan::Pro,
                );
            }
        }
    }

    let notify_payload = if should_notify_for(&payload.status) {
        Some(payload_message(&payload, &state.dashboard_url))
    } else {
        None
    };
    let req = CreateDeploymentRequest::from_payload(payload, user_id.clone());

    match state.deployments_service.create_deployment(&req).await {
        Ok(id) => match state.deployments_service.get_deployment(id, &user_id).await {
            Ok(deployment) => {
                if let Some(message) = notify_payload {
                    notify_user(
                        state.notifier_service.clone(),
                        state.billing_service.clone(),
                        user_id.clone(),
                        message,
                        state.email.clone(),
                    );
                }
                Json(ApiResponse::success(deployment)).into_response()
            }
            Err(e) => {
                error!("Error retrieving created deployment: {e:?}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(e) => {
            error!("Error creating deployment: {e:?}");
            StatusCode::BAD_REQUEST.into_response()
        }
    }
}

fn should_notify_for(status: &DeploymentStatus) -> bool {
    matches!(
        status,
        DeploymentStatus::Success | DeploymentStatus::Failed | DeploymentStatus::RollbackFinished,
    )
}

fn payload_message(payload: &CreateDeployment, dashboard_url: &str) -> Message {
    payload.to_message_with_dashboard(Some(dashboard_url))
}

/// Fan out a deployment-event message to the user's notifiers, in the
/// background. We also pass `billing_service` so notifiers whose kind is
/// no longer allowed under the user's current plan are dropped — without
/// this, paying for a single month of Pro permanently unlocks Slack
/// notifications even after a downgrade.
fn notify_user<NS: NotifierService, BS: BillingService>(
    notifier_service: Arc<NS>,
    billing_service: Arc<BS>,
    user_id: String,
    message: Message,
    email: Option<EmailDispatchConfig>,
) {
    tokio::spawn(async move {
        let plan = match billing_service.get_plan(&user_id).await {
            Ok(p) => p,
            Err(e) => {
                error!("notify_user: plan lookup failed for {user_id}: {e:?}");
                return;
            }
        };
        let limits = plan.limits();
        match notifier_service.list_notifiers(&user_id).await {
            Ok(notifiers) if !notifiers.is_empty() => {
                let allowed: Vec<_> = notifiers
                    .into_iter()
                    .filter(|n| limits.allows_notifier_kind(n.kind))
                    .collect();
                if allowed.is_empty() {
                    return;
                }
                dispatch_to_all(allowed, message, email).await;
            }
            Ok(_) => {}
            Err(e) => error!("Failed to load notifiers for {user_id}: {e:?}"),
        }
    });
}

pub use hoister_shared::wire::PostContainerStateRequest;

async fn post_container_state<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name)): Path<(HostName, ProjectName)>,
    Json(payload): Json<PostContainerStateRequest>,
) -> Response {
    debug!(
        "Received container state update for user: {} host: {} project: {}",
        user_id,
        hostname.as_str(),
        project_name.as_str()
    );

    // Enforce per-plan project cap on first sighting of a new project.
    // We count distinct project names across the user's hosts; mirroring a
    // single compose stack to multiple machines still counts as one project.
    let plan = match state.billing_service.get_plan(&user_id).await {
        Ok(p) => p,
        Err(e) => {
            error!("Failed to fetch plan for {user_id}: {e:?}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if let Some(max) = plan.limits().max_projects {
        let states = state
            .container_state_service
            .get_container_states(&user_id)
            .await;
        let mut distinct: std::collections::HashSet<&ProjectName> =
            states.values().flat_map(|projs| projs.keys()).collect();
        let already_tracked = distinct.contains(&project_name);
        if !already_tracked {
            distinct.insert(&project_name);
            if distinct.len() as i64 > max {
                return upgrade_required(
                    &format!(
                        "Project '{}' would exceed the Free plan limit of {max} projects. Upgrade to Pro for unlimited projects.",
                        project_name.as_str(),
                    ),
                    Plan::Pro,
                );
            }
        }
    }

    let req = AddContainerStateRequest {
        user_id,
        hostname,
        project_name,
        services: payload.payload,
    };
    state.container_state_service.add_container_state(req).await;

    StatusCode::OK.into_response()
}

#[derive(TS, Serialize)]
#[ts(export)]
struct ContainerStateResponse {
    hostname: HostName,
    project_name: ProjectName,
    service_name: ServiceName,
    /// Routed through `serde_json::Value` (BTreeMap-backed by default) so
    /// nested maps — Labels, ExposedPorts, Networks, etc. — serialize in a
    /// stable alphabetical order rather than HashMap's arbitrary one.
    #[ts(type = "any")]
    container_inspections: serde_json::Value,
    last_logs: Option<String>,
    last_updated: DateTime<Utc>,
}

fn inspect_to_sorted_value(
    inspect: &bollard::models::ContainerInspectResponse,
) -> serde_json::Value {
    serde_json::to_value(inspect).unwrap_or(serde_json::Value::Null)
}

#[derive(TS, Serialize)]
#[ts(export)]
struct ContainerStateResponses(Vec<ContainerStateResponse>);

impl From<ContainerStateData> for ContainerStateResponses {
    fn from(value: ContainerStateData) -> Self {
        let mut responses = Vec::new();
        for (hostname, projects) in value.iter() {
            for (project_name, host_project_state) in projects.iter() {
                for (service_name, service_state) in host_project_state.services.iter() {
                    let r = ContainerStateResponse {
                        hostname: hostname.clone(),
                        project_name: project_name.clone(),
                        service_name: service_name.clone(),
                        container_inspections: inspect_to_sorted_value(&service_state.inspect),
                        last_logs: service_state.last_logs.clone(),
                        last_updated: host_project_state.last_updated,
                    };
                    responses.push(r);
                }
            }
        }
        responses.sort_by(|a, b| {
            a.hostname
                .cmp(&b.hostname)
                .then_with(|| a.project_name.0.cmp(&b.project_name.0))
                .then_with(|| a.service_name.0.cmp(&b.service_name.0))
        });
        Self(responses)
    }
}

async fn get_container_state_by_service_name<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name, service_name)): Path<(HostName, ProjectName, ServiceName)>,
) -> Result<Json<ApiResponse<ContainerStateResponse>>, StatusCode> {
    debug!("Received request for container state by id (user: {user_id})");
    let host_project_state = state
        .container_state_service
        .get_container_state(&user_id, &hostname, &project_name, &service_name)
        .await
        .ok_or(StatusCode::NOT_FOUND)?;

    let service_state = host_project_state
        .services
        .into_values()
        .next()
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json::from(ApiResponse::success(ContainerStateResponse {
        hostname,
        project_name,
        service_name,
        container_inspections: inspect_to_sorted_value(&service_state.inspect),
        last_logs: service_state.last_logs,
        last_updated: host_project_state.last_updated,
    })))
}

/// Internal endpoint: drop one (host, project) for the user, freeing a slot
/// against the plan's project cap. Deleting the container_state row cascades
/// (via the `container_metrics → container_state` foreign key) to the
/// project's persisted metrics, so nothing is left behind. The agent
/// recreates the project on its next report if it is still running and
/// labelled — deletion is for projects the user has actually retired.
async fn delete_project<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name)): Path<(HostName, ProjectName)>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state
        .container_state_service
        .delete_project(&user_id, &hostname, &project_name)
        .await;
    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

async fn get_container_states<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> impl IntoResponse {
    debug!("Received request for container states (user: {user_id})");
    let states = state
        .container_state_service
        .get_container_states(&user_id)
        .await;
    Json(ContainerStateResponses::from(states)).into_response()
}

/// Agent endpoint: ingest a batch of resource-usage samples for one
/// (host, project). Opt-in on the agent side, so most agents never call this.
async fn post_container_metrics<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name)): Path<(HostName, ProjectName)>,
    Json(payload): Json<PostContainerMetricsRequest>,
) -> Response {
    debug!(
        "Received {} metric samples for user: {} host: {} project: {}",
        payload.payload.len(),
        user_id,
        hostname.as_str(),
        project_name.as_str()
    );
    let req = AddMetricsRequest::new(user_id, hostname, project_name, payload.payload);
    state.metrics_service.add_metrics(req).await;
    StatusCode::OK.into_response()
}

#[derive(TS, Serialize)]
#[ts(export)]
struct MetricPointResponse {
    recorded_at: DateTime<Utc>,
    cpu_pct: f64,
    // u64 in the domain; JSON-serialized as a plain number. Memory in bytes
    // never approaches 2^53, so `number` is safe and avoids `bigint` in TS.
    #[ts(type = "number")]
    mem_bytes: u64,
    #[ts(type = "number")]
    mem_limit_bytes: u64,
}

#[derive(TS, Serialize)]
#[ts(export)]
struct ServiceMetricsResponse {
    hostname: HostName,
    project_name: ProjectName,
    service_name: ServiceName,
    points: Vec<MetricPointResponse>,
}

#[derive(TS, Serialize)]
#[ts(export)]
struct LatestMetricResponse {
    hostname: HostName,
    project_name: ProjectName,
    service_name: ServiceName,
    recorded_at: DateTime<Utc>,
    cpu_pct: f64,
    #[ts(type = "number")]
    mem_bytes: u64,
    #[ts(type = "number")]
    mem_limit_bytes: u64,
}

#[derive(TS, Serialize)]
#[ts(export)]
struct LatestMetricsResponse(Vec<LatestMetricResponse>);

/// Internal endpoint: time series for one service over the retention window,
/// for the per-container detail graphs.
async fn get_service_metrics<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name, service_name)): Path<(HostName, ProjectName, ServiceName)>,
) -> impl IntoResponse {
    debug!("Received request for service metrics (user: {user_id})");
    let since = Utc::now() - chrono::Duration::days(RETENTION_DAYS);
    let points = state
        .metrics_service
        .get_service_metrics(&user_id, &hostname, &project_name, &service_name, since)
        .await
        .into_iter()
        .map(|p| MetricPointResponse {
            recorded_at: p.recorded_at,
            cpu_pct: p.cpu_pct,
            mem_bytes: p.mem_bytes,
            mem_limit_bytes: p.mem_limit_bytes,
        })
        .collect();
    Json(ApiResponse::success(ServiceMetricsResponse {
        hostname,
        project_name,
        service_name,
        points,
    }))
    .into_response()
}

/// Internal endpoint: latest sample per container for the dashboard aggregate.
async fn get_latest_metrics<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> impl IntoResponse {
    debug!("Received request for latest metrics (user: {user_id})");
    let latest = state
        .metrics_service
        .get_latest_metrics(&user_id)
        .await
        .into_iter()
        .map(|m| LatestMetricResponse {
            hostname: m.hostname,
            project_name: m.project_name,
            service_name: m.service_name,
            recorded_at: m.point.recorded_at,
            cpu_pct: m.point.cpu_pct,
            mem_bytes: m.point.mem_bytes,
            mem_limit_bytes: m.point.mem_limit_bytes,
        })
        .collect();
    Json(LatestMetricsResponse(latest)).into_response()
}

#[derive(Deserialize)]
struct PendingUpdateRequest {
    hostname: HostName,
    project_name: ProjectName,
    service_name: ServiceName,
    image_name: String,
    new_digest: String,
}

async fn post_pending_update<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(payload): Json<PendingUpdateRequest>,
) -> impl IntoResponse {
    let update = PendingUpdate {
        hostname: payload.hostname,
        project_name: payload.project_name,
        service_name: payload.service_name,
        image_name: payload.image_name,
        new_digest: payload.new_digest,
        detected_at: Utc::now(),
    };
    let message = pending_update_message(&update);
    state.pending_updates.add(&user_id, update).await;
    notify_user(
        state.notifier_service.clone(),
        state.billing_service.clone(),
        user_id,
        message,
        state.email.clone(),
    );
    StatusCode::OK.into_response()
}

fn pending_update_message(update: &PendingUpdate) -> Message {
    let title = format!("Update available: {}", update.image_name);
    let body = format!(
        "New image {} ({}) is available\n(project {} | service {} | host {})",
        update.image_name,
        update.new_digest,
        update.project_name.as_str(),
        update.service_name.as_str(),
        update.hostname.as_str(),
    );
    // Share the per-image thread key with deployment-result notifications so an
    // "update available" notice and the success/rollback that follow it land in
    // the same conversation instead of a separate "Update available: …" thread.
    Message::new(title, body).with_subject(deployment_email_subject(
        &update.image_name,
        update.hostname.as_str(),
    ))
}

async fn get_pending_updates<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> impl IntoResponse {
    let updates = state.pending_updates.get_all(&user_id).await;
    Json(updates).into_response()
}

async fn apply_pending_update<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name, service_name)): Path<(HostName, ProjectName, ServiceName)>,
) -> impl IntoResponse {
    state
        .pending_updates
        .remove(&user_id, &hostname, &project_name, &service_name)
        .await;
    let event = ControllerEvent::ApplyUpdate((hostname, project_name, service_name));
    let _ = state.event_tx.send((user_id, event));
    StatusCode::OK.into_response()
}

/// Internal endpoint: delete a user and all their data via CASCADE.
/// Called by the BFF after it has verified a Clerk `user.deleted` webhook.
/// The BFF sets `X-User-Id` to the deleted user's Clerk ID before calling
/// this endpoint, so the identity comes from the trusted internal middleware.
async fn delete_user<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    State(state): State<AppState<DS, CS, TS, NS, BS, MS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> StatusCode {
    if state.billing_service.delete_user(&user_id).await {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::NOT_FOUND
    }
}

/// Agent-facing router: publicly reachable (behind TLS), authenticated via `hst_` tokens.
/// Handles writes from agents and SSE.
pub async fn create_agent_router<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    state: AppState<DS, CS, TS, NS, BS, MS>,
) -> Router {
    let rate_limiter = RateLimiter::new();
    Router::new()
        .route("/health", get(health))
        .route("/sse", get(sse_handler))
        .route(
            "/deployments",
            post(create_deployment::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/state/{hostname}/{project_name}",
            post(post_container_state::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/metrics/{hostname}/{project_name}",
            post(post_container_metrics::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/pending-updates",
            post(post_pending_update::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/pending-updates",
            get(get_pending_updates::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/pending-updates/{hostname}/{project_name}/{service_name}/apply",
            post(apply_pending_update::<DS, CS, TS, NS, BS, MS>),
        )
        // Rate limit runs AFTER auth so it can key on the resolved user_id.
        // Auth runs first because `.layer` applies in reverse order.
        .layer(middleware::from_fn(rate_limit_middleware))
        .layer(Extension(rate_limiter))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            agent_auth_middleware::<DS, CS, TS, NS, BS, MS>,
        ))
        .layer(DefaultBodyLimit::max(AGENT_BODY_LIMIT))
        // Audit log is outermost so it sees the final response status,
        // including 401/429 from the middleware layers above.
        .layer(middleware::from_fn(audit_log_middleware))
        .with_state(state)
}

/// Internal router: VPC-isolated (not publicly reachable), no cryptographic auth.
/// Serves the BFF (SvelteKit frontend-cloud). Trusts `X-User-Id` header.
pub async fn create_internal_router<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
    NS: NotifierService,
    BS: BillingService,
    MS: MetricsService,
>(
    state: AppState<DS, CS, TS, NS, BS, MS>,
    internal_secret: InternalSecret,
) -> Router {
    if internal_secret.0.as_deref().unwrap_or_default().is_empty() {
        log::warn!(
            "Internal router has no X-Internal-Auth secret configured \
             (HOISTER_CONTROLLER_INTERNAL_SECRET). Only safe when the \
             listener binds to loopback — any host or container that can \
             reach the internal port can impersonate any user otherwise."
        );
    }
    Router::new()
        .route("/health", get(health))
        .route("/me", get(get_me::<DS, CS, TS, NS, BS, MS>))
        .route("/tokens", get(list_tokens::<DS, CS, TS, NS, BS, MS>))
        .route("/tokens", post(create_token::<DS, CS, TS, NS, BS, MS>))
        .route(
            "/tokens/{id}",
            axum::routing::delete(delete_token::<DS, CS, TS, NS, BS, MS>),
        )
        .route("/notifiers", get(list_notifiers::<DS, CS, TS, NS, BS, MS>))
        .route(
            "/notifiers",
            post(create_notifier::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/notifiers/{id}",
            axum::routing::delete(delete_notifier::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/notifiers/{id}/enabled",
            axum::routing::patch(set_notifier_enabled::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/notifiers/{id}/test",
            post(test_notifier::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/deployments",
            get(get_deployments::<DS, CS, TS, NS, BS, MS>),
        )
        .route("/billing/plan", post(set_plan::<DS, CS, TS, NS, BS, MS>))
        .route(
            "/deployments/{project_name}/{service_name}",
            get(get_deployments_by_service::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/state",
            get(get_container_states::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/state/{hostname}/{project_name}/{service_name}",
            get(get_container_state_by_service_name::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/state/{hostname}/{project_name}",
            axum::routing::delete(delete_project::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/metrics",
            get(get_latest_metrics::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/container/metrics/{hostname}/{project_name}/{service_name}",
            get(get_service_metrics::<DS, CS, TS, NS, BS, MS>),
        )
        // Pending-update read/apply mirrored from the agent router so the
        // BFF can drive them. Writes (POST /pending-updates) stay agent-only.
        .route(
            "/pending-updates",
            get(get_pending_updates::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/pending-updates/{hostname}/{project_name}/{service_name}/apply",
            post(apply_pending_update::<DS, CS, TS, NS, BS, MS>),
        )
        .route(
            "/users",
            axum::routing::delete(delete_user::<DS, CS, TS, NS, BS, MS>),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            internal_user_middleware::<DS, CS, TS, NS, BS, MS>,
        ))
        .layer(Extension(internal_secret))
        .layer(middleware::from_fn(audit_log_middleware))
        .with_state(state)
}

#[cfg(test)]
mod tests {
    use super::constant_time_eq;

    #[test]
    fn constant_time_eq_matches_strings() {
        assert!(constant_time_eq(b"secret", b"secret"));
        assert!(!constant_time_eq(b"secret", b"Secret"));
        assert!(!constant_time_eq(b"secret", b"secrex"));
    }

    #[test]
    fn constant_time_eq_rejects_length_mismatch() {
        assert!(!constant_time_eq(b"secret", b"secret-longer"));
        assert!(!constant_time_eq(b"", b"a"));
        assert!(constant_time_eq(b"", b""));
    }
}
