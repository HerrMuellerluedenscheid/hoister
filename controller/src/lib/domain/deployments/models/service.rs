use crate::domain::deployments::models::deployment::ProjectId;
use hoister_shared::ServiceName;

pub struct ServiceId(pub i64);

#[allow(dead_code)]
pub struct Service {
    pub(crate) id: ServiceId,
    pub(crate) name: ServiceName,
    pub(crate) project_id: ProjectId,
    pub(crate) created_at: String,
}
