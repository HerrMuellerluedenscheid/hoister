use crate::domain::tokens::models::{ApiToken, TokenError};

pub trait TokenRepository: Send + Sync + 'static + Clone {
    /// List all of `user_id`'s tokens (no plaintext, prefix + comment only).
    fn list_tokens(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<ApiToken>, TokenError>> + Send;

    /// Generate a fresh token, store its hash, return the plaintext exactly
    /// once. `comment` is an optional human-readable label.
    fn create_token(
        &self,
        user_id: &str,
        comment: Option<String>,
    ) -> impl Future<Output = Result<ApiToken, TokenError>> + Send;

    /// Delete a token by id, scoped to `user_id` so users cannot delete
    /// each other's rows. Returns `Ok(true)` if a row was removed.
    fn delete_token(
        &self,
        user_id: &str,
        token_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, TokenError>> + Send;

    /// Look up the Clerk user ID that owns `token`, if any. Used by the
    /// agent-auth middleware.
    fn find_user_by_token(&self, token: &str) -> impl Future<Output = Option<String>> + Send;
}

pub trait TokenService: Send + Sync + 'static + Clone {
    fn list_tokens(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<ApiToken>, TokenError>> + Send;

    fn create_token(
        &self,
        user_id: &str,
        comment: Option<String>,
    ) -> impl Future<Output = Result<ApiToken, TokenError>> + Send;

    fn delete_token(
        &self,
        user_id: &str,
        token_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, TokenError>> + Send;

    fn find_user_by_token(&self, token: &str) -> impl Future<Output = Option<String>> + Send;
}
