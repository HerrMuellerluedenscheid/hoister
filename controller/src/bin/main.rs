use controller::inbound::server::start_server;
use env_logger::Env;
use log::info;

use controller::domain::deployments::service::Service;
use controller::outbound::sqlite::Sqlite;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let port = std::env::var("HOISTER_CONTROLLER_PORT").unwrap_or("3033".to_string());
    let db_path = std::env::var("HOISTER_DATABASE_PATH")
        .expect("HOISTER_DATABASE_PATH must be set (full path to sqlite file)");

    info!("Connecting to database: {db_path}");
    let db = Sqlite::new(&db_path).await?;
    let deployments_service = Service::new(db.clone());

    db.init().await?;
    let auth_token = std::env::var("AUTH_TOKEN").ok();
    info!("Starting server on port {port}");
    start_server(
        auth_token,
        port.parse()
            .expect("Failed to parse HOISTER_CONTROLLER_PORT to integer"),
    )
    .await?;

    Ok(())
}
