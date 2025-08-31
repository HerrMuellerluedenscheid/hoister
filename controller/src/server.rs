use axum::{
    extract::{Path, State, Request},
    http::{StatusCode},
    middleware::{self, Next},
    response::{Json, Response},
    routing::{get, post, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::net::TcpListener;

// Import your database module
use crate::database::{Database, DbError, Deployment};

#[derive(Clone)]
pub struct AppState {
    pub database: Arc<Database>,
    pub api_secret: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateDeployment {
    pub digest: String,
    pub email: String,
}

#[derive(Deserialize)]
pub struct UpdateDeployment {
    pub digest: String,
    pub email: String,
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

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
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
         return Ok(next.run(request).await)
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
            eprintln!("Error getting deployments: {:?}", e);
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
            eprintln!("Error getting deployment {}: {:?}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn create_deployment(
    State(state): State<AppState>,
    Json(payload): Json<CreateDeployment>,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    match state.database.create_deployment(&payload.digest, &payload.email).await {
        Ok(id) => {
            match state.database.get_deployment(id).await {
                Ok(Some(deployment)) => Ok(Json(ApiResponse::success(deployment))),
                Ok(None) => Err(StatusCode::INTERNAL_SERVER_ERROR),
                Err(e) => {
                    eprintln!("Error retrieving created deployment: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Err(e) => {
            eprintln!("Error creating deployment: {:?}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

async fn update_deployment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<UpdateDeployment>,
) -> Result<Json<ApiResponse<Deployment>>, StatusCode> {
    match state.database.update_deployment(id, &payload.digest, &payload.email).await {
        Ok(true) => {
            match state.database.get_deployment(id).await {
                Ok(Some(deployment)) => Ok(Json(ApiResponse::success(deployment))),
                Ok(None) => Err(StatusCode::NOT_FOUND),
                Err(e) => {
                    eprintln!("Error retrieving updated deployment: {:?}", e);
                    Err(StatusCode::INTERNAL_SERVER_ERROR)
                }
            }
        }
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error updating deployment {}: {:?}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn delete_deployment(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.database.delete_deployment(id).await {
        Ok(true) => Ok(Json(ApiResponse::success(()))),
        Ok(false) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            eprintln!("Error deleting deployment {}: {:?}", id, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}


pub async fn create_app(database: Arc<Database>, api_secret: Option<String>) -> Router {
    let state = AppState {
        database,
        api_secret,
    };

    Router::new()
        .route("/health", get(health))
        .route("/deployments", get(get_deployments))
        .route("/deployments", post(create_deployment))
        .route("/deployments/{id}", get(get_deployment))
        .route("/deployments/{id}", delete(delete_deployment))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware))
        .with_state(state)
}

pub async fn start_server(database: Arc<Database>, api_secret: Option<String>, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(database, api_secret).await;
    let listener = TcpListener::bind(format!("0.0.0.0:{}", port)).await?;

    println!("Server running on http://0.0.0.0:{}", port);
    println!("Health check: http://0.0.0.0:{}/health (no auth required)", port);
    println!("Protected API endpoints (require Authorization: Bearer <secret>):");
    println!("  GET    /deployments           - Get all deployments");
    println!("  POST   /deployments           - Create deployment");
    println!("  GET    /deployments/:id       - Get deployment by ID");
    println!("  PUT    /deployments/:id       - Update deployment");
    println!("  DELETE /deployments/:id       - Delete deployment");
    println!("  GET    /deployments/email/:email - Get deployment by email");

    axum::serve(listener, app).await?;
    Ok(())
}