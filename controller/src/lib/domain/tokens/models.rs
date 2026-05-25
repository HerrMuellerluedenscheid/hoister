use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ApiToken {
    /// The plaintext token. Only populated when `is_new == true` — the
    /// controller stores only a SHA-256 hash, so for returning users we
    /// cannot recover the original token.
    pub token: Option<String>,
    pub user_id: String,
    /// True when the token was just created (first login).
    pub is_new: bool,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Unknown error")]
    UnknownError,
}
