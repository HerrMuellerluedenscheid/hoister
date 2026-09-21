use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError, NotifierScope};
use hoister_shared::ProjectName;

pub trait NotifierRepository: Send + Sync + 'static + Clone {
    fn list_notifiers(
        &self,
        scope: NotifierScope<'_>,
    ) -> impl Future<Output = Result<Vec<Notifier>, NotifierError>> + Send;

    /// Every notifier an event of `owner_id`'s project `project_name` goes to:
    /// the owner's account-wide notifiers plus the project's own.
    fn list_event_notifiers(
        &self,
        owner_id: &str,
        project_name: &ProjectName,
    ) -> impl Future<Output = Result<Vec<Notifier>, NotifierError>> + Send;

    /// `project_id` of `None` creates an account-wide notifier. For a project
    /// notifier `user_id` must be the project owner.
    fn create_notifier(
        &self,
        user_id: &str,
        project_id: Option<uuid::Uuid>,
        config: NotifierConfig,
    ) -> impl Future<Output = Result<Notifier, NotifierError>> + Send;

    fn delete_notifier(
        &self,
        scope: NotifierScope<'_>,
        notifier_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;

    fn set_enabled(
        &self,
        scope: NotifierScope<'_>,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;
}

pub trait NotifierService: Send + Sync + 'static + Clone {
    fn list_notifiers(
        &self,
        scope: NotifierScope<'_>,
    ) -> impl Future<Output = Result<Vec<Notifier>, NotifierError>> + Send;

    fn list_event_notifiers(
        &self,
        owner_id: &str,
        project_name: &ProjectName,
    ) -> impl Future<Output = Result<Vec<Notifier>, NotifierError>> + Send;

    fn create_notifier(
        &self,
        user_id: &str,
        project_id: Option<uuid::Uuid>,
        config: NotifierConfig,
    ) -> impl Future<Output = Result<Notifier, NotifierError>> + Send;

    fn delete_notifier(
        &self,
        scope: NotifierScope<'_>,
        notifier_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;

    fn set_enabled(
        &self,
        scope: NotifierScope<'_>,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;
}
