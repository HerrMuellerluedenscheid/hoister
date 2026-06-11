use crate::domain::billing::models::{Plan, PlanError};
use crate::domain::billing::ports::{BillingService, PlanRepository};

#[derive(Clone)]
pub struct Service<PR: PlanRepository> {
    repository: PR,
}

impl<PR: PlanRepository> Service<PR> {
    pub fn new(repository: PR) -> Self {
        Self { repository }
    }
}

impl<PR: PlanRepository> BillingService for Service<PR> {
    async fn get_plan(&self, user_id: &str) -> Result<Plan, PlanError> {
        self.repository.get_plan(user_id).await
    }

    async fn set_plan(&self, user_id: &str, plan: Plan) -> Result<(), PlanError> {
        self.repository.set_plan(user_id, plan).await
    }

    async fn upsert_user(&self, user_id: &str) {
        self.repository.upsert_user(user_id).await
    }

    async fn delete_user(&self, user_id: &str) -> bool {
        self.repository.delete_user(user_id).await
    }
}
