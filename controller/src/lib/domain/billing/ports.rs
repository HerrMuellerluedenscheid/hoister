use crate::domain::billing::models::{Plan, PlanError};

pub trait PlanRepository: Send + Sync + 'static + Clone {
    /// Returns the user's plan. A missing row is treated as Free.
    fn get_plan(&self, user_id: &str) -> impl Future<Output = Result<Plan, PlanError>> + Send;

    /// Upsert the user's plan. Used by the Clerk webhook handler when it
    /// lands (currently called only in tests / admin tooling).
    fn set_plan(
        &self,
        user_id: &str,
        plan: Plan,
    ) -> impl Future<Output = Result<(), PlanError>> + Send;
}

pub trait BillingService: Send + Sync + 'static + Clone {
    fn get_plan(&self, user_id: &str) -> impl Future<Output = Result<Plan, PlanError>> + Send;

    fn set_plan(
        &self,
        user_id: &str,
        plan: Plan,
    ) -> impl Future<Output = Result<(), PlanError>> + Send;
}
