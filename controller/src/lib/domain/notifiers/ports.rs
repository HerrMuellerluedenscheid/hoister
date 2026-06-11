use crate::domain::notifiers::models::{Notifier, NotifierConfig, NotifierError};

pub trait NotifierRepository: Send + Sync + 'static + Clone {
    fn list_notifiers(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Notifier>, NotifierError>> + Send;

    fn create_notifier(
        &self,
        user_id: &str,
        config: NotifierConfig,
    ) -> impl Future<Output = Result<Notifier, NotifierError>> + Send;

    fn delete_notifier(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;

    fn set_enabled(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;
}

pub trait NotifierService: Send + Sync + 'static + Clone {
    fn list_notifiers(
        &self,
        user_id: &str,
    ) -> impl Future<Output = Result<Vec<Notifier>, NotifierError>> + Send;

    fn create_notifier(
        &self,
        user_id: &str,
        config: NotifierConfig,
    ) -> impl Future<Output = Result<Notifier, NotifierError>> + Send;

    fn delete_notifier(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;

    fn set_enabled(
        &self,
        user_id: &str,
        notifier_id: uuid::Uuid,
        enabled: bool,
    ) -> impl Future<Output = Result<bool, NotifierError>> + Send;
}
