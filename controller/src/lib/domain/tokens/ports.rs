use crate::domain::tokens::models::{ApiToken, TokenError};

pub trait TokenRepository: Send + Sync + 'static + Clone {
    /// Return the existing token for `user_id`, or generate and persist a new one.
    fn get_or_create_token(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<ApiToken, TokenError>> + Send;

    /// Replace the user's stored token with a freshly-generated one. The
    /// plaintext is returned once; subsequent calls to `get_or_create_token`
    /// will not be able to recover it.
    fn rotate_token(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<ApiToken, TokenError>> + Send;

    /// Look up the Clerk user ID that owns `token`, if any.
    fn find_user_by_token(&self, token: &str) -> impl Future<Output = Option<String>> + Send;
}

pub trait TokenService: Send + Sync + 'static + Clone {
    fn get_or_create_token(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<ApiToken, TokenError>> + Send;

    fn rotate_token(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<ApiToken, TokenError>> + Send;

    fn find_user_by_token(&self, token: &str) -> impl Future<Output = Option<String>> + Send;
}
