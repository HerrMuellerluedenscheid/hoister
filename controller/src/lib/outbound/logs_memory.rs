use chrono::{DateTime, Utc};
use hoister_shared::{HostName, ProjectName, ServiceName};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// How long a forwarded log tail is kept before it is treated as stale and
/// dropped. On-demand logs are throwaway: the dashboard fetches them right after
/// requesting, so a short window keeps memory bounded while tolerating the
/// round-trip to the agent and back.
const LOG_TTL_SECS: i64 = 300;

type LogsStore = HashMap<(HostName, ProjectName, ServiceName), LogEntry>;

/// One on-demand log tail plus when the controller received it.
#[derive(Clone, Serialize)]
pub struct LogEntry {
    pub logs: String,
    pub received_at: DateTime<Utc>,
}

/// In-memory, user-partitioned store of on-demand container logs.
///
/// Deliberately NOT backed by the database: logs can contain secrets that
/// keyword redaction won't catch, and they are throwaway — the dashboard pulls
/// them moments after requesting. Entries expire after `LOG_TTL_SECS`.
#[derive(Clone, Default)]
pub struct LogsMemory {
    logs: Arc<RwLock<HashMap<String, LogsStore>>>,
}

impl LogsMemory {
    /// Store the latest log tail for one service, replacing any previous one.
    pub async fn set(
        &self,
        user_id: &str,
        hostname: HostName,
        project: ProjectName,
        service: ServiceName,
        logs: String,
    ) {
        let entry = LogEntry {
            logs,
            received_at: Utc::now(),
        };
        let mut guard = self.logs.write().await;
        let store = guard.entry(user_id.to_string()).or_default();
        prune(store);
        store.insert((hostname, project, service), entry);
    }

    /// Fetch the latest non-expired log tail for one service, if any.
    pub async fn get(
        &self,
        user_id: &str,
        hostname: &HostName,
        project: &ProjectName,
        service: &ServiceName,
    ) -> Option<LogEntry> {
        let mut guard = self.logs.write().await;
        let store = guard.get_mut(user_id)?;
        prune(store);
        store
            .get(&(hostname.clone(), project.clone(), service.clone()))
            .cloned()
    }
}

/// Drop entries older than the TTL. Cheap: called on each access, and the map
/// only holds the handful of services a user has recently inspected.
fn prune(store: &mut LogsStore) {
    let cutoff = Utc::now() - chrono::Duration::seconds(LOG_TTL_SECS);
    store.retain(|_, entry| entry.received_at > cutoff);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> (HostName, ProjectName, ServiceName) {
        (
            HostName::new("host"),
            ProjectName::new("proj"),
            ServiceName::new("svc"),
        )
    }

    #[tokio::test]
    async fn set_then_get_roundtrips() {
        let mem = LogsMemory::default();
        let (h, p, s) = key();
        mem.set("user", h.clone(), p.clone(), s.clone(), "hello".into())
            .await;
        let got = mem.get("user", &h, &p, &s).await.expect("entry present");
        assert_eq!(got.logs, "hello");
    }

    #[tokio::test]
    async fn get_is_scoped_per_user() {
        let mem = LogsMemory::default();
        let (h, p, s) = key();
        mem.set("alice", h.clone(), p.clone(), s.clone(), "secret".into())
            .await;
        assert!(mem.get("bob", &h, &p, &s).await.is_none());
    }

    #[tokio::test]
    async fn expired_entries_are_pruned() {
        let mem = LogsMemory::default();
        let (h, p, s) = key();
        // Insert a manually back-dated entry to simulate TTL expiry.
        {
            let mut guard = mem.logs.write().await;
            let store = guard.entry("user".to_string()).or_default();
            store.insert(
                (h.clone(), p.clone(), s.clone()),
                LogEntry {
                    logs: "old".into(),
                    received_at: Utc::now() - chrono::Duration::seconds(LOG_TTL_SECS + 1),
                },
            );
        }
        assert!(mem.get("user", &h, &p, &s).await.is_none());
    }
}
