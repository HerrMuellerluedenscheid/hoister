use hoister_shared::ServiceName;
use crate::domain::deployments::models::deployment::ProjectId;

pub struct ServiceId(pub i64);

pub struct Service {
    pub(crate) id: ServiceId,
    pub(crate) name: ServiceName,
    pub(crate) project_id: ProjectId,
    pub(crate) created_at: String,
}