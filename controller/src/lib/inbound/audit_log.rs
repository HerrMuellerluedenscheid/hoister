//! Per-request audit log middleware.
//!
//! Emits one structured info line per authenticated request, carrying the
//! resolved user_id (or `-` for unauthenticated paths like /health) along
//! with the method, path, status, and elapsed time. Useful for incident
//! response — when something looks wrong on a user's account, grep the
//! log for their user_id.

use axum::{Extension, extract::Request, middleware::Next, response::Response};
use log::info;
use std::time::Instant;

use crate::inbound::server::UserId;

pub async fn audit_log_middleware(
    user: Option<Extension<UserId>>,
    request: Request,
    next: Next,
) -> Response {
    let started = Instant::now();
    let method = request.method().clone();
    let path = request
        .uri()
        .path_and_query()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    let response = next.run(request).await;

    let user_id = user
        .as_ref()
        .map(|Extension(UserId(id))| id.as_str())
        .unwrap_or("-");
    let status = response.status().as_u16();
    let elapsed_ms = started.elapsed().as_millis();

    info!(target: "hoister.audit", "user={user_id} method={method} path={path} status={status} elapsed_ms={elapsed_ms}");

    response
}
