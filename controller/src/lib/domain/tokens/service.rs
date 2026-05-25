use crate::domain::tokens::models::{ApiToken, TokenError};
use crate::domain::tokens::ports::{TokenRepository, TokenService};

#[derive(Clone)]
pub struct Service<TR: TokenRepository> {
    repository: TR,
}

impl<TR: TokenRepository> Service<TR> {
    pub fn new(repository: TR) -> Self {
        Self { repository }
    }
}

impl<TR: TokenRepository> TokenService for Service<TR> {
    async fn get_or_create_token(&self, user_id: &str) -> Result<ApiToken, TokenError> {
        self.repository.get_or_create_token(user_id).await
    }

    async fn rotate_token(&self, user_id: &str) -> Result<ApiToken, TokenError> {
        self.repository.rotate_token(user_id).await
    }

    async fn find_user_by_token(&self, token: &str) -> Option<String> {
        self.repository.find_user_by_token(token).await
    }
}
