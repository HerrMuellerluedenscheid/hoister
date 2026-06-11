use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError};
use crate::domain::notifiers::ports::{NotifierRepository, NotifierService};

#[derive(Clone)]
pub struct Service<NR: NotifierRepository> {
    repository: NR,
}

impl<NR: NotifierRepository> Service<NR> {
    pub fn new(repository: NR) -> Self {
        Self { repository }
    }
}

impl<NR: NotifierRepository> NotifierService for Service<NR> {
    async fn list_notifiers(&self, user_id: &str) -> Result<Vec<Notifier>, NotifierError> {
        self.repository.list_notifiers(user_id).await
    }

    async fn create_notifier(
        &self,
        user_id: &str,
        config: NotifierConfig,
    ) -> Result<Notifier, NotifierError> {
        self.repository.create_notifier(user_id, config).await
    }

    async fn delete_notifier(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
    ) -> Result<bool, NotifierError> {
        self.repository.delete_notifier(user_id, notifier_id).await
    }

    async fn set_enabled(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<bool, NotifierError> {
        self.repository
            .set_enabled(user_id, notifier_id, enabled)
            .await
    }
}
