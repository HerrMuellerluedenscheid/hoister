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
use log::{debug, error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

use crate::domain::deployments::models::deployment::{
    CreateDeploymentRequest, Deployment, GetDeploymentError,
};
use crate::domain::deployments::ports::{DeploymentsRepository, DeploymentsService};
use crate::domain::deployments::service::Service;
use crate::sse::{ControllerEvent, sse_handler};
use hoister_shared::{CreateDeployment, ProjectName, ServiceName};
use tokio::sync::{RwLock, broadcast};
use ts_rs::TS;

type ContainerState = HashMap<ProjectName, HashMap<ServiceName, ContainerInspectResponse>>;

#[derive(Clone)]
pub struct AppState<DS: DeploymentsService> {
    pub deployments_service: Arc<DS>,
    pub container_state: Arc<RwLock<ContainerState>>,
    pub api_secret: Option<String>,
    pub event_tx: broadcast::Sender<ControllerEvent>,
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

// Authentication middleware
async fn auth_middleware<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
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

async fn get_deployments<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    match state.deployments_service.get_all_deployments().await {
        Ok(deployments) => Ok(Json(ApiResponse::success(deployments))),
        Err(e) => {
            error!("Error getting deployments: {e:?}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_deployments_by_service<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
    Path((project_name, service_name)): Path<(ProjectName, ServiceName)>,
) -> Result<Json<ApiResponse<Vec<Deployment>>>, StatusCode> {
    debug!("get service by name: {:?}", service_name);

    match state
        .deployments_service
        .get_deployments_of_service(&project_name, &service_name)
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

async fn create_deployment<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
    Json(payload): Json<CreateDeployment>,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    match state
        .deployments_service
        .create_deployment(&CreateDeploymentRequest::from(payload))
        .await
    {
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
    pub payload: HashMap<ServiceName, ContainerInspectResponse>,
}

async fn post_container_state<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
    Json(payload): Json<PostContainerStateRequest>,
) -> impl IntoResponse {
    debug!(
        "Received container state update for project: {}",
        payload.project_name.as_str()
    );
    let mut lock = state.container_state.write().await;
    lock.insert(payload.project_name, payload.payload);
    StatusCode::OK.into_response()
}

#[derive(TS, Serialize)]
#[ts(export)]
struct ContainerStateResponse {
    project_name: ProjectName,
    service_name: ServiceName,
    #[ts(type = "any")]
    container_inspections: ContainerInspectResponse,
}

#[derive(TS, Serialize)]
#[ts(export)]
struct ContainerStateResponses(Vec<ContainerStateResponse>);

async fn get_container_state_by_service_name<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
    Path((project_name, service_name)): Path<(ProjectName, ServiceName)>,
) -> Result<Json<ApiResponse<ContainerStateResponse>>, StatusCode> {
    debug!("Received request for container state by id");
    let container_state = state.container_state.read().await;
    let response = container_state
        .get(&project_name)
        .and_then(|x| x.get(&service_name))
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(Json::from(ApiResponse::success(ContainerStateResponse {
        project_name: project_name.clone(),
        service_name: service_name.clone(),
        container_inspections: response.clone(),
    })))
}

async fn get_container_states<DS: DeploymentsService>(
    State(state): State<AppState<DS>>,
) -> impl IntoResponse {
    debug!("Received request for container state");
    let container_state = state.container_state.read().await;
    debug!("Sending container state: {:?}", container_state);
    let mut states: Vec<ContainerStateResponse> = Vec::new();
    for (project_name, services) in container_state.iter() {
        for (service_name, container_inspection) in services.iter() {
            debug!(
                "Sending container state for project: {} service: {}",
                project_name.as_str(),
                service_name.as_str()
            );
            states.push(ContainerStateResponse {
                project_name: project_name.clone(),
                service_name: service_name.clone(),
                container_inspections: container_inspection.clone(),
            });
        }
    }

    Json(ContainerStateResponses(states)).into_response()
}

pub async fn create_app<DR: DeploymentsRepository>(state: AppState<Service<DR>>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/sse", get(sse_handler))
        .route("/deployments", get(get_deployments))
        .route("/deployments", post(create_deployment))
        .route(
            "/deployments/{project_name}/{service_name}",
            get(get_deployments_by_service),
        )
        .route(
            "/container/state/{project_name}",
            post(post_container_state),
        )
        .route("/container/state", get(get_container_states))
        .route(
            "/container/state/{project_name}/{service_name}",
            get(get_container_state_by_service_name),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .with_state(state)
}
