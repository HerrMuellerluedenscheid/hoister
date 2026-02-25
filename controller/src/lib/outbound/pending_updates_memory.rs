use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type UpdatesStore = HashMap<(HostName, ProjectName, ServiceName), PendingUpdate>;

#[derive(Clone, Serialize)]
pub struct PendingUpdate {
    pub hostname: HostName,
    pub project_name: ProjectName,
    pub service_name: ServiceName,
    pub image_name: String,
    pub new_digest: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Clone, Default)]
pub struct PendingUpdatesMemory {
    updates: Arc<RwLock<UpdatesStore>>,
}

impl PendingUpdatesMemory {
    pub async fn add(&self, update: PendingUpdate) {
        let key = (
            update.hostname.clone(),
            update.project_name.clone(),
            update.service_name.clone(),
        );
        self.updates.write().await.insert(key, update);
    }

    pub async fn get_all(&self) -> Vec<PendingUpdate> {
        self.updates.read().await.values().cloned().collect()
    }

    pub async fn remove(&self, hostname: &HostName, project: &ProjectName, service: &ServiceName) {
        let key = (hostname.clone(), project.clone(), service.clone());
        self.updates.write().await.remove(&key);
    }
}
