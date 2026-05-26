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
    async fn list_tokens(&self, user_id: &str) -> Result<Vec<ApiToken>, TokenError> {
        self.repository.list_tokens(user_id).await
    }

    async fn create_token(
        &self,
        user_id: &str,
        comment: Option<String>,
    ) -> Result<ApiToken, TokenError> {
        self.repository.create_token(user_id, comment).await
    }

    async fn delete_token(&self, user_id: &str, token_id: i64) -> Result<bool, TokenError> {
        self.repository.delete_token(user_id, token_id).await
    }

    async fn find_user_by_token(&self, token: &str) -> Option<String> {
        self.repository.find_user_by_token(token).await
    }
}
