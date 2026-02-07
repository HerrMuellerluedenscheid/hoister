use controller::config::get_config;
use controller::domain::deployments::service::Service;
use controller::inbound::server::{AppState, create_app};
use controller::outbound::sqlite::Sqlite;
use controller::sse::ControllerEvent;
use env_logger::Env;
use log::info;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{RwLock, broadcast};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = get_config();

    let (event_tx, _) = broadcast::channel::<ControllerEvent>(100);
    let repo = Sqlite::new(&config.database_path)
        .await
        .expect("Failed to connect to database");
    let deployments_service = Service::new(repo);

    let state = AppState {
        deployments_service: Arc::new(deployments_service),
        container_state: Arc::new(RwLock::new(HashMap::new())),
        api_secret: config.api_secret.clone(),
        event_tx,
    };

    let app = create_app(state).await;
    let listener = TcpListener::bind(format!("0.0.0.0:{}", config.port)).await?;

    info!("Server running on http://0.0.0.0:{}", config.port);
    info!(
        "Health check: http://0.0.0.0:{}/health (no auth required)",
        config.port
    );
    info!("Protected API endpoints (require Authorization: Bearer <secret>):");
    info!("  GET    /sse                   - server side events");
    info!("  GET    /deployments           - Get all deployments");
    info!("  POST   /deployments           - Create deployment");
    info!("  GET    /deployments/:id       - Get deployment by ID");
    info!("  PUT    /deployments/:id       - Update deployment");

    axum::serve(listener, app).await?;

    Ok(())
}
