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

/// In-memory pending-update store, partitioned by user_id so one user's
/// pending updates are never visible to another.
#[derive(Clone, Default)]
pub struct PendingUpdatesMemory {
    updates: Arc<RwLock<HashMap<String, UpdatesStore>>>,
}

impl PendingUpdatesMemory {
    pub async fn add(&self, user_id: &str, update: PendingUpdate) {
        let key = (
            update.hostname.clone(),
            update.project_name.clone(),
            update.service_name.clone(),
        );
        self.updates
            .write()
            .await
            .entry(user_id.to_string())
            .or_default()
            .insert(key, update);
    }

    pub async fn get_all(&self, user_id: &str) -> Vec<PendingUpdate> {
        self.updates
            .read()
            .await
            .get(user_id)
            .map(|store| store.values().cloned().collect())
            .unwrap_or_default()
    }

    pub async fn remove(
        &self,
        user_id: &str,
        hostname: &HostName,
        project: &ProjectName,
        service: &ServiceName,
    ) {
        let key = (hostname.clone(), project.clone(), service.clone());
        if let Some(store) = self.updates.write().await.get_mut(user_id) {
            store.remove(&key);
        }
    }
}
