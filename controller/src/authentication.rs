use std::fs::File;
use std::io::{BufReader, Read};
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::Response;
use http::{HeaderMap, StatusCode};

#[cfg(feature = "jwt_auth")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "jwt_auth")]
use jwt_simple::prelude::*;

use crate::database::DataStore;
use crate::server::AppState;

// Authentication middleware
pub async fn auth_middleware<T: DataStore>(
    State(state): State<AppState<T>>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {

    // disable auth if jwt_auth feature is enabled
    #[cfg(feature = "jwt_auth")]
    {
        return Ok(next.run(request).await);
    }
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


#[cfg(feature = "jwt_auth")]
#[derive(Serialize, Deserialize)]
pub struct ProjectClaim {
    pub user_id: String,
    pub project: String,
}

#[cfg(feature = "jwt_auth")]
pub fn read_key() -> HS256Key {
    let key_file_path = std::env::var("HOISTER_JWT_KEY_FILE_PATH").expect("HOISTER_JWT_KEY_FILE_PATH not set");
    let f = File::open(key_file_path).expect("error reading key file");
    let mut reader = BufReader::new(f);
    let mut buffer = String::new();
    reader
        .read_to_string(&mut buffer)
        .expect("error reading key file");
    let key = HS256Key::from_bytes((&buffer).as_ref());

    key
}

#[cfg(feature = "jwt_auth")]
pub fn validate_jwt(token: &str) -> Option<ProjectClaim> {
    let key = read_key();
    match key.verify_token::<ProjectClaim>(token, None) {
        Ok(claims) => Some(claims.custom),
        Err(_) => None,
    }
}


pub fn extract_token_header(headers: HeaderMap) -> Option<ProjectClaim> {
    // Get JWT from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))?;
    let claim = validate_jwt(token)?;
    Some(claim)
}
