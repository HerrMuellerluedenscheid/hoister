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
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::container_state::models::state::{
    AddContainerStateRequest, ContainerStateData, ServiceState,
};
use crate::domain::container_state::port::ContainerStateService;
use crate::domain::deployments::models::deployment::{
    CreateDeploymentRequest, Deployment, GetDeploymentError,
};
use crate::domain::deployments::ports::DeploymentsService;
use crate::domain::tokens::models::ApiToken;
use crate::domain::tokens::ports::TokenService;
use crate::outbound::pending_updates_memory::{PendingUpdate, PendingUpdatesMemory};
use crate::sse::{ControllerEvent, UserScopedEvent, sse_handler};
use hoister_shared::{CreateDeployment, HostName, ProjectName, ServiceName};
use tokio::sync::broadcast;
use ts_rs::TS;

/// user ID extracted from a verified request, injected as a request extension.
/// For internal (BFF) requests this comes from the `X-User-Id` header (trusted because
/// the internal router is VPC-isolated, not publicly reachable).
/// For agent requests this comes from the per-user token DB lookup.
#[derive(Debug, Clone)]
pub struct UserId(pub String);

#[derive(Clone)]
pub struct AppState<DS: DeploymentsService, CS: ContainerStateService, TS: TokenService> {
    pub deployments_service: Arc<DS>,
    pub container_state_service: Arc<CS>,
    pub token_service: Arc<TS>,
    #[cfg(feature = "self-hosted")]
    pub api_secret: Option<String>,
    pub event_tx: broadcast::Sender<UserScopedEvent>,
    pub pending_updates: PendingUpdatesMemory,
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
>(
    State(state): State<AppState<DS, CS, TS>>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Self-hosted: if no secret is configured, skip auth entirely (open dev mode).
    // Insert a synthetic "local" UserId so downstream handlers can rely on
    // tenant scoping being non-optional.
    #[cfg(feature = "self-hosted")]
    if state.api_secret.is_none() {
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

    // 1. Static API secret (self-hosted only). The X-User-Id header is
    // optional; absent it we scope to the synthetic "local" tenant.
    #[cfg(feature = "self-hosted")]
    if let Some(ref secret) = state.api_secret {
        if token == *secret {
            let user_id = request
                .headers()
                .get("X-User-Id")
                .and_then(|h| h.to_str().ok())
                .map(str::to_owned)
                .unwrap_or_else(|| "local".to_string());
            request.extensions_mut().insert(UserId(user_id));
            return Ok(next.run(request).await);
        }
        return Ok(next.run(request).await);
    }

    // 2. Per-user agent token (hst_ prefix) — look up user ID from DB.
    if token.starts_with("hst_") {
        match state.token_service.find_user_by_token(&token).await {
            Some(user_id) => {
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
async fn internal_user_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
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
    request.extensions_mut().insert(UserId(user_id));

    Ok(next.run(request).await)
}

// Web handlers

async fn health() -> &'static str {
    "OK"
}

async fn get_or_create_token<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    State(state): State<AppState<DS, CS, TS>>,
    user: Extension<UserId>,
) -> Result<Json<ApiResponse<ApiToken>>, StatusCode> {
    let Extension(UserId(user_id)) = user;
    match state.token_service.get_or_create_token(&user_id).await {
        Ok(token) => Ok(Json(ApiResponse::success(token))),
        Err(e) => {
            error!("Error getting/creating token for {user_id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_deployments<DS: DeploymentsService, CS: ContainerStateService, TS: TokenService>(
    State(state): State<AppState<DS, CS, TS>>,
    user: Option<Extension<UserId>>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    let user_id = user.map(|Extension(UserId(id))| id);
    match state
        .deployments_service
        .get_all_deployments(user_id.as_deref())
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
>(
    State(state): State<AppState<DS, CS, TS>>,
    Path((project_name, service_name)): Path<(ProjectName, ServiceName)>,
    user: Option<Extension<UserId>>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    debug!("get service by name: {service_name:?}");
    let user_id = user.map(|Extension(UserId(id))| id);
    match state
        .deployments_service
        .get_deployments_of_service(&project_name, &service_name, user_id.as_deref())
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

async fn create_deployment<DS: DeploymentsService, CS: ContainerStateService, TS: TokenService>(
    State(state): State<AppState<DS, CS, TS>>,
    user: Option<Extension<UserId>>,
    Json(payload): Json<CreateDeployment>,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    let user_id = user.map(|Extension(UserId(id))| id);
    let mut req = CreateDeploymentRequest::from(payload);
    req.user_id = user_id;

    match state.deployments_service.create_deployment(&req).await {
        Ok(id) => match state.deployments_service.get_deployment(id).await {
            Ok(deployment) => Ok(Json(ApiResponse::success(deployment))),
            Err(e) => {
                eprintln!("Error retrieving created deployment: {e:?}");
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
        Err(e) => {
            eprintln!("Error creating deployment: {e:?}");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct PostContainerStateRequest {
    pub project_name: ProjectName,
    pub payload: HashMap<ServiceName, ServiceState>,
}

async fn post_container_state<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    State(state): State<AppState<DS, CS, TS>>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((hostname, project_name)): Path<(HostName, ProjectName)>,
    Json(payload): Json<PostContainerStateRequest>,
) -> impl IntoResponse {
    debug!(
        "Received container state update for user: {} host: {} project: {}",
        user_id,
        hostname.as_str(),
        project_name.as_str()
    );

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
    #[ts(type = "any")]
    container_inspections: bollard::models::ContainerInspectResponse,
    last_logs: Option<String>,
    last_updated: DateTime<Utc>,
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
                        container_inspections: service_state.inspect.clone(),
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
>(
    State(state): State<AppState<DS, CS, TS>>,
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
        container_inspections: service_state.inspect,
        last_logs: service_state.last_logs,
        last_updated: host_project_state.last_updated,
    })))
}

async fn get_container_states<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    State(state): State<AppState<DS, CS, TS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> impl IntoResponse {
    debug!("Received request for container states (user: {user_id})");
    let states = state
        .container_state_service
        .get_container_states(&user_id)
        .await;
    Json(ContainerStateResponses::from(states)).into_response()
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
>(
    State(state): State<AppState<DS, CS, TS>>,
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
    state.pending_updates.add(&user_id, update).await;
    StatusCode::OK.into_response()
}

async fn get_pending_updates<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    State(state): State<AppState<DS, CS, TS>>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> impl IntoResponse {
    let updates = state.pending_updates.get_all(&user_id).await;
    Json(updates).into_response()
}

async fn apply_pending_update<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    State(state): State<AppState<DS, CS, TS>>,
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

/// Agent-facing router: publicly reachable (behind TLS), authenticated via `hst_` tokens.
/// Handles writes from agents and SSE.
pub async fn create_agent_router<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    state: AppState<DS, CS, TS>,
) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/sse", get(sse_handler))
        .route("/deployments", post(create_deployment::<DS, CS, TS>))
        .route(
            "/container/state/{hostname}/{project_name}",
            post(post_container_state::<DS, CS, TS>),
        )
        .route("/pending-updates", post(post_pending_update::<DS, CS, TS>))
        .route("/pending-updates", get(get_pending_updates::<DS, CS, TS>))
        .route(
            "/pending-updates/{hostname}/{project_name}/{service_name}/apply",
            post(apply_pending_update::<DS, CS, TS>),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            agent_auth_middleware::<DS, CS, TS>,
        ))
        .with_state(state)
}

/// Internal router: VPC-isolated (not publicly reachable), no cryptographic auth.
/// Serves the BFF (SvelteKit frontend-cloud). Trusts `X-User-Id` header.
pub async fn create_internal_router<
    DS: DeploymentsService,
    CS: ContainerStateService,
    TS: TokenService,
>(
    state: AppState<DS, CS, TS>,
) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/token", get(get_or_create_token::<DS, CS, TS>))
        .route("/deployments", get(get_deployments::<DS, CS, TS>))
        .route(
            "/deployments/{project_name}/{service_name}",
            get(get_deployments_by_service::<DS, CS, TS>),
        )
        .route("/container/state", get(get_container_states::<DS, CS, TS>))
        .route(
            "/container/state/{hostname}/{project_name}/{service_name}",
            get(get_container_state_by_service_name::<DS, CS, TS>),
        )
        .layer(middleware::from_fn(internal_user_middleware))
        .with_state(state)
}
