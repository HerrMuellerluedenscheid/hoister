use axum::{
    Router,
    extract::{Path, Request, State},
    http::StatusCode,
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post},
};
use log::info;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::sync::Arc;
use http::HeaderMap;
use tokio::net::TcpListener;

// Import your database module
use crate::database::DataStore;
use crate::database::{LocalSQLite, Deployment};
use crate::sse::{ControllerEvent, sse_handler};
use crate::authentication::{auth_middleware, extract_token_header};
use sqlx::Type;
use tokio::sync::broadcast;
use ts_rs::TS;

#[derive(Clone)]
pub struct AppState<T: DataStore> {
    pub(crate) database: Arc<T>,
    pub(crate) api_secret: Option<String>,
    pub(crate) event_tx: broadcast::Sender<ControllerEvent>,
}

#[derive(TS)]
#[ts(export)]
#[derive(Deserialize, Debug, Clone, Serialize, Type)]
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

// Web handlers
async fn health() -> &'static str {
    "OK"
}

async fn get_deployments<T: DataStore>(
    State(state): State<AppState<T>>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    match state.database.get_all_deployment().await {
        Ok(deployments) => Ok(Json(ApiResponse::success(deployments))),
        Err(e) => {
            eprintln!("Error getting deployments: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_deployment<T: DataStore>(
    State(state): State<AppState<T>>,
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

async fn create_deployment<T: DataStore>(
    State(state): State<AppState<T>>,
    Json(payload): Json<CreateDeployment>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    let claim = extract_token_header(headers).ok_or(StatusCode::UNAUTHORIZED)?;
    match state
        .database
        .create_deployment(&payload.image, &payload.status, claim)
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

pub async fn create_app(database: Arc<LocalSQLite>, api_secret: Option<String>) -> Router {
    let (event_tx, _) = broadcast::channel::<ControllerEvent>(100);

    let state = AppState {
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
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}

pub async fn start_server(
    database: Arc<LocalSQLite>,
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
