use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ApiToken {
    pub id: i64,
    pub user_id: String,
    /// Plaintext token. Only populated on creation; subsequent reads only
    /// have access to the hash, so this field is `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
    /// First 12 chars of the token (`hst_<8 hex>`). Stored separately so the
    /// dashboard can show a stable, displayable identifier without the
    /// plaintext.
    pub token_prefix: String,
    /// Optional human-readable label set when the token was created (e.g.
    /// "vectorandveneer", "ci-runner-eu-1").
    pub comment: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Token not found")]
    NotFound,
    #[error("Unknown error")]
    UnknownError,
}
