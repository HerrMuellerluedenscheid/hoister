use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use std::collections::HashMap;

pub use hoister_shared::wire::ServiceState;

pub struct AddContainerStateRequest {
    /// Owning user. In hosted mode this is the Clerk user id resolved from the
    /// agent's `hst_` token; in self-hosted dev mode it falls back to "local".
    pub(crate) user_id: String,
    pub(crate) hostname: HostName,
    pub(crate) project_name: ProjectName,
    pub(crate) services: HashMap<ServiceName, ServiceState>,
}

#[derive(Clone)]
pub struct HostProjectState {
    pub services: HashMap<ServiceName, ServiceState>,
    pub last_updated: DateTime<Utc>,
}

/// One user's view of container state. The repository stores a separate copy
/// of this per user so reads can never leak across tenants.
pub(crate) type ContainerStateData = HashMap<HostName, HashMap<ProjectName, HostProjectState>>;
