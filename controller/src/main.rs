use std::sync::Arc;
use crate::database::Database;
use crate::server::{start_server};

mod database;
mod server;

// Example usage
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Ensure we can write to current directory
    let db_path = "sqlite:./database.db";
    println!("Connecting to database: {}", db_path);
    let db = Database::new("sqlite:///tmp/sqlite.db").await?;
    let db = Arc::new(db);

    // Initialize the database
    db.init().await?;
    let auth_token = std::env::var("AUTH_TOKEN").ok();
    // Start the web server
    start_server(db, auth_token, 3000).await?;

    Ok(())
}