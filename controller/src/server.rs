use axum::response::IntoResponse;
use axum::{
    Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post},
};
use bollard::models::ContainerInspectResponse;
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use tokio::net::TcpListener;

// Import your database module
use crate::database::{Database, Deployment};
use crate::sse::{ControllerEvent, sse_handler};
use sqlx::Type;
use tokio::sync::{RwLock, broadcast};
use ts_rs::TS;

#[derive(Clone)]
pub struct AppState {
    container_state: Arc<RwLock<Option<Vec<ContainerInspectResponse>>>>,
    pub(crate) database: Arc<Database>,
    pub(crate) api_secret: Option<String>,
    pub(crate) event_tx: broadcast::Sender<ControllerEvent>,
}

#[derive(TS, Deserialize, Debug, Clone, Serialize, Type)]
#[ts(export)]
#[repr(u8)]
pub enum DeploymentStatus {
    Pending = 0,
    Started = 1,
    Success = 2,
    RollbackFinished = 3,
    NoUpdate = 4,
    Failed = 5,
    TestMessage = 6,
}

impl Display for DeploymentStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DeploymentStatus::Pending => write!(f, "Pending"),
            DeploymentStatus::Started => write!(f, "Started"),
            DeploymentStatus::Success => write!(f, "Success"),
            DeploymentStatus::RollbackFinished => write!(f, "Rolled back"),
            &DeploymentStatus::NoUpdate => write!(f, "NoUpdate"),
            &DeploymentStatus::Failed => write!(f, "Failed"),
            &DeploymentStatus::TestMessage => write!(f, "Test Message"),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct CreateDeployment {
    pub image: String,
    pub container_id: String,
    pub status: DeploymentStatus,
}

#[derive(Serialize)]
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

// Authentication middleware
async fn auth_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check
    let api_secret = match state.api_secret {
        Some(secret) => secret,
        None => return Ok(next.run(request).await),
    };
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    };
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let token = &header[7..]; // Remove "Bearer " prefix
            if token == api_secret {
                Ok(next.run(request).await)
            } else {
                Err(StatusCode::UNAUTHORIZED)
            }
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}

// Web handlers
async fn health() -> &'static str {
    "OK"
}

async fn get_deployments(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    match state.database.get_all_deployment().await {
        Ok(deployments) => Ok(Json(ApiResponse::success(deployments))),
        Err(e) => {
            eprintln!("Error getting deployments: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_deployment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    match state.database.get_deployment(id).await {
        Ok(Some(deployment)) => Ok(Json(ApiResponse::success(deployment))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error getting deployment {id}: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_deployment(
    State(state): State<AppState>,
    Json(payload): Json<CreateDeployment>,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    match state
        .database
        .create_deployment(&payload.image, &payload.status)
        .await
    {
        Ok(id) => match state.database.get_deployment(id).await {
            Ok(Some(deployment)) => Ok(Json(ApiResponse::success(deployment))),
            Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
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

async fn post_container_state(
    State(state): State<AppState>,
    Json(payload): Json<Vec<ContainerInspectResponse>>,
) -> impl IntoResponse {
    debug!("Received container state update");
    *state.container_state.write().await = Some(payload);
    StatusCode::OK.into_response()
}

#[derive(Serialize)]
struct ContainerStateResponse {
    container_inspections: Vec<ContainerInspectResponse>,
}

async fn get_container_state_by_id(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    debug!("Received request for container state by id");
    let container_state = state.container_state.read().await;
    match container_state.as_ref() {
        Some(cs) => {
            let cs = cs.clone();
            let container_inspections = cs
                .iter()
                .filter(|x| x.id == Some(id.clone()))
                .cloned()
                .collect::<Vec<ContainerInspectResponse>>();
            let response = ContainerStateResponse {
                container_inspections,
            };
            Json(ApiResponse::success(response)).into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn get_container_state(State(state): State<AppState>) -> impl IntoResponse {
    debug!("Received request for container state");
    let container_state = state.container_state.read().await;
    match container_state.as_ref() {
        Some(cs) => {
            let response = ContainerStateResponse {
                container_inspections: cs.clone(),
            };
            Json(ApiResponse::success(response)).into_response()
        }
        None => {
            info!("No container state received, yet");
            Json(ApiResponse::success(ContainerStateResponse {
                container_inspections: vec![],
            }))
            .into_response()
        }
    }
}

pub async fn create_app(database: Arc<Database>, api_secret: Option<String>) -> Router {
    let (event_tx, _) = broadcast::channel::<ControllerEvent>(100);

    let state = AppState {
        container_state: Arc::new(RwLock::new(None)),
        database,
        api_secret,
        event_tx,
    };

    Router::new()
        .route("/health", get(health))
        .route("/sse", get(sse_handler))
        .route("/deployments", get(get_deployments))
        .route("/deployments", post(create_deployment))
        .route("/deployments/{id}", get(get_deployment))
        .route("/container/state", post(post_container_state))
        .route("/container/state", get(get_container_state))
        .route("/container/state/{id}", get(get_container_state_by_id))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

pub async fn start_server(
    database: Arc<Database>,
    api_secret: Option<String>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(database, api_secret).await;
    let listener = TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    info!("Server running on http://0.0.0.0:{port}");
    info!("Health check: http://0.0.0.0:{port}/health (no auth required)");
    info!("Protected API endpoints (require Authorization: Bearer <secret>):");
    info!("  GET    /sse                   - server side events");
    info!("  GET    /deployments           - Get all deployments");
    info!("  POST   /deployments           - Create deployment");
    info!("  GET    /deployments/:id       - Get deployment by ID");
    info!("  PUT    /deployments/:id       - Update deployment");

    axum::serve(listener, app).await?;
    Ok(())
}
