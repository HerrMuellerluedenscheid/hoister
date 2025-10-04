use crate::database::Database;
use crate::server::start_server;
use env_logger::Env;
use log::info;
use std::sync::Arc;

mod database;
mod server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let port = std::env::var("HOISTER_CONTROLLER_PORT").unwrap_or("3003".to_string());
    let db_path = std::env::var("HOISTER_DATABASE_PATH")
        .expect("HOISTER_DATABASE_PATH must be set (full path to sqlite file)");

    info!("Connecting to database: {db_path}");
    let db = Database::new(&db_path).await?;
    let db = Arc::new(db);

    db.init().await?;
    let auth_token = std::env::var("AUTH_TOKEN").ok();
    info!("Starting server on port {port}");
    start_server(
        db,
        auth_token,
        port.parse()
            .expect("Failed to parse HOISTER_CONTROLLER_PORT to integer"),
    )
    .await?;

    Ok(())
}
