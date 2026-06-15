use crate::HoisterError;
use crate::docker::get_service_identifier;
use bollard::Docker;
use bollard::models::{ContainerInspectResponse, ContainerStateStatusEnum, ContainerSummary};
use bollard::query_parameters::{ListContainersOptions, LogsOptionsBuilder};
use futures_util::StreamExt;
use hoister_shared::wire::{PostContainerStateRequest, ServiceState};
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::{debug, error, info, warn};
use reqwest::Url;
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;
use tokio::time;

/// Substring placed in env-var values and log output by the agent before
/// anything leaves the host. Kept in sync with the frontend's `REDACTION_MARKER`
/// (frontend/frontend-cloud `src/lib/redaction.ts`) so the UI can render it as a
/// badge instead of literal asterisks.
pub(crate) const REDACTION_MARKER: &str = "***REDACTED***";

/// Built-in case-insensitive substrings that mark an env-var key as sensitive.
/// Operators extend this at runtime via `redact_keywords` in the config; see
/// [`init_extra_keywords`].
const DEFAULT_SENSITIVE_KEYWORDS: &[&str] = &[
    "telegram_chat_id",
    "discord_channel_id",
    "slack_webhook",
    "password",
    "passwd",
    "pwd",
    "secret",
    "token",
    "key",
    "auth",
    "credential",
    "cred",
    "apikey",
    "api_key",
    "username",
    "user",
    "session",
    "cookie",
];

/// Operator-supplied extra keywords (already lower-cased), set once at startup
/// from config. Empty until [`init_extra_keywords`] runs, so redaction always
/// falls back to the built-in list.
static EXTRA_SENSITIVE_KEYWORDS: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();

/// Register the operator-supplied redaction keywords loaded from config. Called
/// once at startup; later calls are ignored. Entries are trimmed, lower-cased,
/// and empties dropped so matching stays case-insensitive like the built-ins.
pub(crate) fn init_extra_keywords(keywords: Vec<String>) {
    let normalised = keywords
        .into_iter()
        .map(|k| k.trim().to_lowercase())
        .filter(|k| !k.is_empty())
        .collect();
    let _ = EXTRA_SENSITIVE_KEYWORDS.set(normalised);
}

/// True when the (already lower-cased) env-var key contains any sensitive
/// keyword, built-in or operator-supplied.
fn key_is_sensitive(key_lower: &str) -> bool {
    DEFAULT_SENSITIVE_KEYWORDS
        .iter()
        .any(|keyword| key_lower.contains(keyword))
        || EXTRA_SENSITIVE_KEYWORDS
            .get()
            .is_some_and(|extra| extra.iter().any(|keyword| key_lower.contains(keyword)))
}

/// Max bytes of container log tail we ship to the controller. Cap is intentional:
/// crash-loop logs are usually short, and we don't want a chatty container to
/// blow up the request payload.
const MAX_LOG_BYTES: usize = 16 * 1024;
const LOG_TAIL_LINES: &str = "50";

/// List the containers this agent should report on: scoped to the compose
/// project (in release builds) and with any `hoister.hide=true` containers
/// excluded. Shared by the state monitor and the metrics collector so the
/// two always observe the same set of containers.
pub(crate) async fn list_tracked_containers(
    #[allow(unused_variables)] project_name: &ProjectName,
    docker: &Docker,
) -> Result<Vec<ContainerSummary>, HoisterError> {
    #[allow(unused_mut, unused_variables)]
    let mut filters = HashMap::new();
    #[cfg(not(debug_assertions))]
    {
        let label_filters = vec![format!(
            "com.docker.compose.project={}",
            project_name.as_str()
        )];
        filters.insert("label".to_string(), label_filters);
    }
    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;

    Ok(containers
        .into_iter()
        .filter(|container| {
            if let Some(labels) = &container.labels {
                // Exclude if hoister.hide is explicitly set to "true"
                !matches!(labels.get("hoister.hide"), Some(val) if val == "true")
            } else {
                true
            }
        })
        .collect::<Vec<ContainerSummary>>())
}

async fn fetch_container_info(
    project_name: &ProjectName,
    docker: &Docker,
    report_logs: bool,
) -> Result<HashMap<ServiceName, ServiceState>, HoisterError> {
    let containers = list_tracked_containers(project_name, docker).await?;

    let mut states = HashMap::new();

    for container in containers {
        if let Some(container_id) = &container.id {
            let service_identifier = get_service_identifier(docker, container_id).await?;

            let inspect = docker
                .inspect_container(
                    container_id,
                    None::<bollard::query_parameters::InspectContainerOptions>,
                )
                .await;

            match inspect {
                Ok(mut inspect) => {
                    // Fetch logs BEFORE redacting env vars so we can use the
                    // original sensitive values to scrub them from log output.
                    // Disabled unless the operator explicitly opted in:
                    // logs can contain secrets that keyword-based redaction
                    // doesn't catch.
                    let last_logs = if report_logs && should_fetch_logs(&inspect) {
                        match fetch_log_tail(docker, container_id, &inspect, 0).await {
                            Ok(logs) => logs,
                            Err(e) => {
                                warn!("Failed to fetch logs for {container_id}: {e}");
                                None
                            }
                        }
                    } else {
                        None
                    };

                    redact_credentials(&mut inspect);
                    strip_health_check_output(&mut inspect);
                    prune_inspect(&mut inspect);
                    states.insert(
                        service_identifier.clone(),
                        ServiceState { inspect, last_logs },
                    );
                }
                Err(e) => error!("Error inspecting container {container_id}: {e}"),
            }
        }
    }

    Ok(states)
}

/// Only attach a log tail when the container is in a state where logs explain
/// something the user can't otherwise see. For running/created containers logs
/// are noise.
fn should_fetch_logs(inspect: &ContainerInspectResponse) -> bool {
    let Some(state) = &inspect.state else {
        return false;
    };
    if state.restarting == Some(true) {
        return true;
    }
    matches!(
        state.status,
        Some(ContainerStateStatusEnum::RESTARTING)
            | Some(ContainerStateStatusEnum::EXITED)
            | Some(ContainerStateStatusEnum::DEAD)
    )
}

/// Tail a container's logs (≤16 KB / 50 lines) and redact any values of
/// sensitive env vars from `inspect`. Shared with the rollback path in
/// `docker.rs`, which captures the failed container's logs before removing it.
///
/// `since` is a Unix timestamp (seconds, `i32` as bollard requires); when set,
/// only log lines emitted at
/// or after it are returned. The rollback path uses this to fetch *only* the
/// restored container's fresh post-restart output, since that container is the
/// long-lived original and its tail would otherwise include stale pre-update
/// lines. `0` returns the full tail.
pub(crate) async fn fetch_log_tail(
    docker: &Docker,
    container_id: &str,
    inspect: &ContainerInspectResponse,
    since: i32,
) -> Result<Option<String>, HoisterError> {
    let options = LogsOptionsBuilder::new()
        .stdout(true)
        .stderr(true)
        .tail(LOG_TAIL_LINES)
        .timestamps(true)
        .follow(false)
        .since(since)
        .build();

    let mut stream = docker.logs(container_id, Some(options));
    let mut buf: Vec<u8> = Vec::new();

    while let Some(chunk) = stream.next().await {
        match chunk {
            Ok(out) => {
                let bytes = out.into_bytes();
                let remaining = MAX_LOG_BYTES.saturating_sub(buf.len());
                if remaining == 0 {
                    break;
                }
                let take = bytes.len().min(remaining);
                buf.extend_from_slice(&bytes[..take]);
                if buf.len() >= MAX_LOG_BYTES {
                    break;
                }
            }
            Err(e) => {
                warn!("Error reading log chunk for {container_id}: {e}");
                break;
            }
        }
    }

    if buf.is_empty() {
        return Ok(None);
    }

    let mut text = String::from_utf8_lossy(&buf).into_owned();
    let sensitive_values = collect_sensitive_env_values(inspect);
    redact_values(&mut text, &sensitive_values);
    Ok(Some(text))
}

/// Pull the *values* of any env vars whose key looks sensitive. We use these
/// to scrub the same secrets out of the log output, in case the application
/// inside the container logged them.
fn collect_sensitive_env_values(inspect: &ContainerInspectResponse) -> Vec<String> {
    let Some(config) = inspect.config.as_ref() else {
        return Vec::new();
    };
    let Some(env) = config.env.as_ref() else {
        return Vec::new();
    };

    env.iter()
        .filter_map(|var| {
            let (key, value) = var.split_once('=')?;
            if value.is_empty() {
                return None;
            }
            let key_lower = key.to_lowercase();
            key_is_sensitive(&key_lower).then(|| value.to_string())
        })
        .collect()
}

/// Replace any occurrence of `needles` in `haystack` with `***REDACTED***`.
/// Replaces longest first so that overlapping secrets get caught fully.
fn redact_values(haystack: &mut String, needles: &[String]) {
    let mut sorted: Vec<&String> = needles.iter().collect();
    sorted.sort_by_key(|s| std::cmp::Reverse(s.len()));
    for needle in sorted {
        if needle.len() < 4 {
            // Short values produce too many false-positive matches.
            continue;
        }
        if haystack.contains(needle.as_str()) {
            *haystack = haystack.replace(needle.as_str(), REDACTION_MARKER);
        }
    }
}

/// Drop the per-probe health-check log before the inspect leaves the host. The
/// dashboard only shows `Health.Status` and `Health.FailingStreak`, so the probe
/// log is dead weight; clearing it also keeps probe output (which can echo back
/// response bodies or secrets the health-check command printed) off the wire.
fn strip_health_check_output(inspect: &mut ContainerInspectResponse) {
    if let Some(state) = inspect.state.as_mut()
        && let Some(health) = state.health.as_mut()
    {
        health.log = None;
    }
}

/// Drop inspect fields the controller and dashboard never read before the
/// payload leaves the host. This shrinks the request and — more importantly —
/// removes large, map-heavy sub-objects (notably `HostConfig`) that bloat every
/// poll, so the diff/heartbeat compression in [`start`] has less surface to
/// churn on.
///
/// The **keep-list** is everything currently consumed downstream:
///   - top level: `Id`, `Created`, `State`, `Image`, `Name`, `RestartCount`,
///     `Mounts`, `Config`, `NetworkSettings`
///   - `Config`: `Labels`, `Image`, `Cmd`, `WorkingDir`, `Hostname`, `Env`
///     (the controller reads `Config.Image`; the dashboard reads the rest)
///   - `State` and `NetworkSettings` are kept whole — they're small and stable.
///
/// If the dashboard or controller starts reading a new field, add it back here.
fn prune_inspect(inspect: &mut ContainerInspectResponse) {
    inspect.path = None;
    inspect.args = None;
    inspect.resolv_conf_path = None;
    inspect.hostname_path = None;
    inspect.hosts_path = None;
    inspect.log_path = None;
    inspect.driver = None;
    inspect.platform = None;
    inspect.image_manifest_descriptor = None;
    inspect.mount_label = None;
    inspect.process_label = None;
    inspect.app_armor_profile = None;
    inspect.exec_ids = None;
    inspect.host_config = None;
    inspect.graph_driver = None;
    inspect.size_rw = None;
    inspect.size_root_fs = None;

    if let Some(config) = inspect.config.as_mut() {
        // Unused maps; dropping them also keeps key-order churn off the wire.
        config.exposed_ports = None;
        config.volumes = None;
    }
}

fn redact_credentials(inspect: &mut ContainerInspectResponse) {
    if let Some(config) = inspect.config.as_mut()
        && let Some(env_vars) = config.env.as_mut()
    {
        *env_vars = env_vars
            .iter()
            .map(|env_var| {
                // Split on first '=' to get key=value
                if let Some((key, _value)) = env_var.split_once('=') {
                    let key_lower = key.to_lowercase();

                    // Check if the key contains any sensitive keyword
                    if key_is_sensitive(&key_lower) {
                        format!("{key}={REDACTION_MARKER}")
                    } else {
                        env_var.clone()
                    }
                } else {
                    env_var.clone()
                }
            })
            .collect();
    }
}

async fn send_to_backend(
    client: &reqwest::Client,
    controller_url: &Url,
    token: Option<&str>,
    project_name: &ProjectName,
    hostname: &HostName,
    body: Vec<u8>,
) -> Result<(), reqwest::Error> {
    let url = controller_url
        .join(format!("container/state/{}/{}", hostname.0, project_name.0).as_str())
        .expect("failed to join url");

    let mut req = client
        .post(url)
        .header("Content-Type", "application/json")
        .body(body);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let response = req.send().await?;
    response.error_for_status()?;
    Ok(())
}

async fn send_heartbeat(
    client: &reqwest::Client,
    controller_url: &Url,
    token: Option<&str>,
    project_name: &ProjectName,
    hostname: &HostName,
) -> Result<(), reqwest::Error> {
    let url = controller_url
        .join(
            format!(
                "container/state/{}/{}/heartbeat",
                hostname.0, project_name.0
            )
            .as_str(),
        )
        .expect("failed to join url");

    let mut req = client.post(url);
    if let Some(token) = token {
        req = req.bearer_auth(token);
    }
    let response = req.send().await?;
    response.error_for_status()?;
    Ok(())
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

pub(crate) async fn start(
    controller_url: &Url,
    token: Option<String>,
    project_name: ProjectName,
    hostname: HostName,
    client: reqwest::Client,
    report_logs: bool,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    info!(
        "Starting monitor (log forwarding: {})",
        if report_logs { "enabled" } else { "disabled" }
    );
    let docker = Docker::connect_with_socket_defaults()?;
    let mut interval = time::interval(Duration::from_secs(60));
    let mut prev_hash: Option<u64> = None;

    loop {
        interval.tick().await;

        match fetch_container_info(&project_name, &docker, report_logs).await {
            Ok(current_states) => {
                let request = PostContainerStateRequest {
                    project_name: project_name.clone(),
                    payload: current_states,
                };
                // Route through `serde_json::Value` (a sorted `BTreeMap`, since
                // serde_json is built without `preserve_order`) so object keys
                // are emitted in a stable order. bollard deserializes inspect
                // fields like `Config.Labels` and `NetworkSettings.Networks`
                // into `HashMap`s whose iteration order is randomized per
                // instance; serializing the struct directly would reshuffle keys
                // on every poll, changing the hash even when nothing changed and
                // defeating the diff/heartbeat compression below.
                let body = match serde_json::to_value(&request)
                    .and_then(|value| serde_json::to_vec(&value))
                {
                    Ok(b) => b,
                    Err(e) => {
                        error!("Failed to serialize state: {e}");
                        continue;
                    }
                };
                let hash = hash_bytes(&body);
                if prev_hash == Some(hash) {
                    debug!("State unchanged, sending heartbeat");
                    if let Err(e) = send_heartbeat(
                        &client,
                        controller_url,
                        token.as_deref(),
                        &project_name,
                        &hostname,
                    )
                    .await
                    {
                        error!("Failed to send heartbeat: {e}");
                    }
                    continue;
                }
                if let Err(e) = send_to_backend(
                    &client,
                    controller_url,
                    token.as_deref(),
                    &project_name,
                    &hostname,
                    body,
                )
                .await
                {
                    error!("Failed to send to backend: {e}");
                } else {
                    prev_hash = Some(hash);
                    debug!(
                        "Successfully sent {} containers to backend",
                        request.payload.len()
                    );
                }
            }
            Err(e) => error!("Error fetching container info: {e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bollard::models::ContainerConfig;

    fn inspect_with_env(env: Vec<&str>) -> ContainerInspectResponse {
        ContainerInspectResponse {
            config: Some(ContainerConfig {
                env: Some(env.into_iter().map(String::from).collect()),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    #[test]
    fn redact_values_replaces_secret_substring() {
        let inspect = inspect_with_env(vec![
            "API_KEY=super-secret-token-12345",
            "BENIGN=some-public-value",
        ]);
        let values = collect_sensitive_env_values(&inspect);
        let mut log = String::from(
            "2024-01-01 starting up\nfailed auth: super-secret-token-12345\ngoodbye\n",
        );
        redact_values(&mut log, &values);
        assert!(!log.contains("super-secret-token-12345"));
        assert!(log.contains(REDACTION_MARKER));
    }

    #[test]
    fn redact_values_skips_short_secrets() {
        // 3-char secret would otherwise match every 3-letter word in a log.
        let inspect = inspect_with_env(vec!["API_TOKEN=abc"]);
        let values = collect_sensitive_env_values(&inspect);
        let mut log = String::from("abc def\n");
        redact_values(&mut log, &values);
        assert_eq!(log, "abc def\n");
    }

    #[test]
    fn collect_sensitive_env_values_ignores_benign_keys() {
        let inspect = inspect_with_env(vec!["PORT=8080", "HOSTNAME=foo"]);
        assert!(collect_sensitive_env_values(&inspect).is_empty());
    }

    #[test]
    fn strip_health_check_output_drops_log_keeps_status() {
        use bollard::models::{ContainerState, Health, HealthStatusEnum, HealthcheckResult};

        let mut inspect = ContainerInspectResponse {
            state: Some(ContainerState {
                health: Some(Health {
                    status: Some(HealthStatusEnum::UNHEALTHY),
                    failing_streak: Some(3),
                    log: Some(vec![HealthcheckResult {
                        exit_code: Some(1),
                        output: Some("HTTP/1.1 500 ... full body".to_string()),
                        ..Default::default()
                    }]),
                }),
                ..Default::default()
            }),
            ..Default::default()
        };

        strip_health_check_output(&mut inspect);

        let health = inspect.state.unwrap().health.unwrap();
        assert_eq!(health.status, Some(HealthStatusEnum::UNHEALTHY));
        assert_eq!(health.failing_streak, Some(3));
        assert_eq!(health.log, None);
    }

    #[test]
    fn key_is_sensitive_matches_builtin_and_extra_keywords() {
        // Built-in keyword matches regardless of operator config.
        assert!(key_is_sensitive("database_password"));
        // An otherwise-benign key only matches once it's registered as a custom
        // keyword. `init_extra_keywords` sets a process-global OnceLock, so this
        // also implicitly covers the case-insensitive normalisation.
        assert!(!key_is_sensitive("acme_license_serial"));
        init_extra_keywords(vec!["  LICENSE_Serial ".to_string(), String::new()]);
        assert!(key_is_sensitive("acme_license_serial"));
    }
}
