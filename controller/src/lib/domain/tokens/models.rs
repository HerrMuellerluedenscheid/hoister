use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Clone, Serialize)]
pub struct ApiToken {
    pub token: String,
    pub user_id: String,
    /// True when the token was just created (first login).
    pub is_new: bool,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("Unknown error")]
    UnknownError,
}
