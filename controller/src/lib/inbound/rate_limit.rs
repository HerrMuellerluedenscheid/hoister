//! Per-user-id token-bucket rate limiter for the agent router.
//!
//! Lives in-process; restarts reset the counters. That's good enough for
//! the volumes we expect (agents posting state every 5 s) — the goal here
//! is shedding abusive load, not enforcing exact quotas.

use axum::{
    Extension,
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

use crate::inbound::server::UserId;

/// Tokens added per second. With a 5-second agent post interval we want
/// some headroom for retries.
const REFILL_PER_SECOND: f64 = 1.0;
/// Maximum number of tokens a single user can accumulate (i.e. burst size).
const BUCKET_CAPACITY: f64 = 60.0;

struct Bucket {
    tokens: f64,
    last_refill: Instant,
}

#[derive(Clone, Default)]
pub struct RateLimiter {
    buckets: Arc<Mutex<HashMap<String, Bucket>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Try to consume one token for `user_id`. Returns true if the request
    /// should be allowed, false if the user is over their limit.
    async fn try_acquire(&self, user_id: &str) -> bool {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().await;
        let bucket = buckets.entry(user_id.to_string()).or_insert(Bucket {
            tokens: BUCKET_CAPACITY,
            last_refill: now,
        });

        let elapsed = now.duration_since(bucket.last_refill).as_secs_f64();
        bucket.tokens = (bucket.tokens + elapsed * REFILL_PER_SECOND).min(BUCKET_CAPACITY);
        bucket.last_refill = now;

        if bucket.tokens >= 1.0 {
            bucket.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

pub async fn rate_limit_middleware(
    Extension(rate_limiter): Extension<RateLimiter>,
    user: Option<Extension<UserId>>,
    request: Request,
    next: Next,
) -> Response {
    // No UserId means the auth middleware didn't run (e.g. /health). Pass.
    let Some(Extension(UserId(user_id))) = user else {
        return next.run(request).await;
    };

    if rate_limiter.try_acquire(&user_id).await {
        next.run(request).await
    } else {
        (StatusCode::TOO_MANY_REQUESTS, "rate limit exceeded").into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn allows_first_request() {
        let rl = RateLimiter::new();
        assert!(rl.try_acquire("alice").await);
    }

    #[tokio::test]
    async fn rejects_after_burst() {
        let rl = RateLimiter::new();
        // Drain the bucket.
        for _ in 0..(BUCKET_CAPACITY as usize) {
            assert!(rl.try_acquire("alice").await);
        }
        // Next request in the same instant should fail.
        assert!(!rl.try_acquire("alice").await);
    }

    #[tokio::test]
    async fn buckets_are_independent_per_user() {
        let rl = RateLimiter::new();
        for _ in 0..(BUCKET_CAPACITY as usize) {
            assert!(rl.try_acquire("alice").await);
        }
        // bob still has a full bucket
        assert!(rl.try_acquire("bob").await);
    }
}
