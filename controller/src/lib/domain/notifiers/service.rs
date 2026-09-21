use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError, NotifierScope};
use crate::domain::notifiers::ports::{NotifierRepository, NotifierService};
use hoister_shared::ProjectName;

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
    async fn list_notifiers(
        &self,
        scope: NotifierScope<'_>,
    ) -> Result<Vec<Notifier>, NotifierError> {
        self.repository.list_notifiers(scope).await
    }

    async fn list_event_notifiers(
        &self,
        owner_id: &str,
        project_name: &ProjectName,
    ) -> Result<Vec<Notifier>, NotifierError> {
        self.repository
            .list_event_notifiers(owner_id, project_name)
            .await
    }

    async fn create_notifier(
        &self,
        user_id: &str,
        project_id: Option<uuid::Uuid>,
        config: NotifierConfig,
    ) -> Result<Notifier, NotifierError> {
        self.repository
            .create_notifier(user_id, project_id, config)
            .await
    }

    async fn delete_notifier(
        &self,
        scope: NotifierScope<'_>,
        notifier_id: uuid::Uuid,
    ) -> Result<bool, NotifierError> {
        self.repository.delete_notifier(scope, notifier_id).await
    }

    async fn set_enabled(
        &self,
        scope: NotifierScope<'_>,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> Result<bool, NotifierError> {
        self.repository
            .set_enabled(scope, notifier_id, enabled)
            .await
    }
}
