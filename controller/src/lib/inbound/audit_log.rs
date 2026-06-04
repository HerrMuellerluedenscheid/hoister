//! Per-request audit span.
//!
//! Opens a tracing span for every request carrying the resolved user_id (or
//! `-` for unauthenticated paths like /health) together with the method and
//! path. Because the span wraps the downstream handler, every log/trace event
//! emitted while serving the request inherits these fields — so an error
//! logged deep inside a service still tells you which user it belonged to.
//! On completion it emits one summary event with the status and elapsed time.

use axum::{Extension, extract::Request, middleware::Next, response::Response};
use std::time::Instant;
use tracing::Instrument;

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

    let user_id = user
        .as_ref()
        .map(|Extension(UserId(id))| id.clone())
        .unwrap_or_else(|| "-".to_string());

    // The span stays enabled at `info` so its fields decorate every child
    // event, but it emits no line of its own. The per-request summary below is
    // `debug`, so the steady stream of requests stays quiet unless opted into.
    let span = tracing::info_span!(
        target: "hoister.audit",
        "request",
        %user_id,
        %method,
        %path,
    );

    let response = next.run(request).instrument(span.clone()).await;

    let status = response.status().as_u16();
    let elapsed_ms = started.elapsed().as_millis();

    let _guard = span.enter();
    tracing::debug!(target: "hoister.audit", status, elapsed_ms, "request completed");

    response
}
