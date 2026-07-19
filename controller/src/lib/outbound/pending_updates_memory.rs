use chrono::{DateTime, Duration, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

type UpdatesStore = HashMap<(HostName, ProjectName, ServiceName), PendingUpdate>;

/// Default eviction TTL. Agents re-report a still-available update on every
/// check tick, refreshing `detected_at`, but the tick is user-configured and
/// may be a daily cron — so the default has to be generous enough to bridge
/// two daily runs with margin.
pub const DEFAULT_PENDING_UPDATE_TTL_SECS: u64 = 172_800;

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
///
/// Entries whose `detected_at` was not refreshed within the TTL are evicted
/// on the read and write paths. Without this, an entry whose key stops
/// matching reality — hostname changed, service removed from the compose
/// file, agent decommissioned — would linger until the next controller
/// restart (see issue #113).
#[derive(Clone)]
pub struct PendingUpdatesMemory {
    updates: Arc<RwLock<HashMap<String, UpdatesStore>>>,
    ttl: Duration,
}

impl Default for PendingUpdatesMemory {
    fn default() -> Self {
        Self::new(Duration::seconds(DEFAULT_PENDING_UPDATE_TTL_SECS as i64))
    }
}

impl PendingUpdatesMemory {
    pub fn new(ttl: Duration) -> Self {
        Self {
            updates: Default::default(),
            ttl,
        }
    }

    fn evict_expired(&self, store: &mut UpdatesStore) {
        let cutoff = Utc::now() - self.ttl;
        store.retain(|_, update| update.detected_at >= cutoff);
    }

    pub async fn add(&self, user_id: &str, update: PendingUpdate) {
        let key = (
            update.hostname.clone(),
            update.project_name.clone(),
            update.service_name.clone(),
        );
        let mut updates = self.updates.write().await;
        let store = updates.entry(user_id.to_string()).or_default();
        store.insert(key, update);
        self.evict_expired(store);
    }

    pub async fn get_all(&self, user_id: &str) -> Vec<PendingUpdate> {
        let mut updates = self.updates.write().await;
        let Some(store) = updates.get_mut(user_id) else {
            return Vec::new();
        };
        self.evict_expired(store);
        store.values().cloned().collect()
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

    /// Drop every pending update belonging to one (host, project). Called when
    /// the user deletes the project from the dashboard so its updates don't
    /// outlive it.
    pub async fn remove_project(&self, user_id: &str, hostname: &HostName, project: &ProjectName) {
        if let Some(store) = self.updates.write().await.get_mut(user_id) {
            store.retain(|(h, p, _), _| !(h == hostname && p == project));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn update(host: &str, project: &str, service: &str, age: Duration) -> PendingUpdate {
        PendingUpdate {
            hostname: HostName::new(host),
            project_name: ProjectName::new(project),
            service_name: ServiceName::new(service),
            image_name: "img:latest".to_string(),
            new_digest: "sha256:abc".to_string(),
            detected_at: Utc::now() - age,
        }
    }

    #[tokio::test]
    async fn expired_entries_are_evicted_on_read() {
        let memory = PendingUpdatesMemory::new(Duration::seconds(60));
        memory
            .add("u1", update("old-host", "p", "s", Duration::seconds(120)))
            .await;
        memory
            .add("u1", update("host", "p", "s", Duration::zero()))
            .await;

        let updates = memory.get_all("u1").await;
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].hostname.as_str(), "host");
    }

    #[tokio::test]
    async fn fresh_entries_survive() {
        let memory = PendingUpdatesMemory::new(Duration::seconds(60));
        memory
            .add("u1", update("host", "p", "s", Duration::seconds(30)))
            .await;
        assert_eq!(memory.get_all("u1").await.len(), 1);
    }

    #[tokio::test]
    async fn re_reporting_refreshes_an_entry() {
        let memory = PendingUpdatesMemory::new(Duration::seconds(60));
        memory
            .add("u1", update("host", "p", "s", Duration::seconds(50)))
            .await;
        // The agent's next check tick re-posts the same update with a fresh
        // detected_at, replacing the aging entry under the same key.
        memory
            .add("u1", update("host", "p", "s", Duration::zero()))
            .await;
        assert_eq!(memory.get_all("u1").await.len(), 1);
    }

    #[tokio::test]
    async fn remove_project_only_drops_matching_entries() {
        let memory = PendingUpdatesMemory::default();
        memory
            .add("u1", update("host", "p1", "s1", Duration::zero()))
            .await;
        memory
            .add("u1", update("host", "p1", "s2", Duration::zero()))
            .await;
        memory
            .add("u1", update("host", "p2", "s1", Duration::zero()))
            .await;

        memory
            .remove_project("u1", &HostName::new("host"), &ProjectName::new("p1"))
            .await;

        let updates = memory.get_all("u1").await;
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].project_name.as_str(), "p2");
    }
}
