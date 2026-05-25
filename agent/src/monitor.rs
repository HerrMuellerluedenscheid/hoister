use crate::HoisterError;
use crate::docker::get_service_identifier;
use bollard::Docker;
use bollard::models::{ContainerInspectResponse, ContainerStateStatusEnum, ContainerSummary};
use bollard::query_parameters::{ListContainersOptions, LogsOptionsBuilder};
use controller::domain::container_state::models::state::ServiceState;
use controller::inbound::server::PostContainerStateRequest;
use futures_util::StreamExt;
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::{debug, error, info, warn};
use reqwest::Url;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

const SENSITIVE_KEYWORDS: &[&str] = &[
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

/// Max bytes of container log tail we ship to the controller. Cap is intentional:
/// crash-loop logs are usually short, and we don't want a chatty container to
/// blow up the request payload.
const MAX_LOG_BYTES: usize = 16 * 1024;
const LOG_TAIL_LINES: &str = "50";

async fn fetch_container_info(
    #[allow(unused_variables)] project_name: &ProjectName,
    docker: &Docker,
) -> Result<HashMap<ServiceName, ServiceState>, HoisterError> {
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

    let containers = containers
        .into_iter()
        .filter(|container| {
            if let Some(labels) = &container.labels {
                // Exclude if hoister.hide is explicitly set to "true"
                !matches!(labels.get("hoister.hide"), Some(val) if val == "true")
            } else {
                true
            }
        })
        .collect::<Vec<ContainerSummary>>();

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
                    let last_logs = if should_fetch_logs(&inspect) {
                        match fetch_log_tail(docker, container_id, &inspect).await {
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

async fn fetch_log_tail(
    docker: &Docker,
    container_id: &str,
    inspect: &ContainerInspectResponse,
) -> Result<Option<String>, HoisterError> {
    let options = LogsOptionsBuilder::new()
        .stdout(true)
        .stderr(true)
        .tail(LOG_TAIL_LINES)
        .timestamps(true)
        .follow(false)
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
            let is_sensitive = SENSITIVE_KEYWORDS
                .iter()
                .any(|keyword| key_lower.contains(keyword));
            is_sensitive.then(|| value.to_string())
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
            *haystack = haystack.replace(needle.as_str(), "***REDACTED***");
        }
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
                    let is_sensitive = SENSITIVE_KEYWORDS
                        .iter()
                        .any(|keyword| key_lower.contains(keyword));

                    if is_sensitive {
                        format!("{key}=***REDACTED***")
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
    project_name: ProjectName,
    hostname: HostName,
    states: &HashMap<ServiceName, ServiceState>,
) -> Result<(), reqwest::Error> {
    let url = controller_url
        .join(format!("container/state/{}/{}", hostname.0, project_name.0).as_str())
        .expect("failed to join url");

    let request = PostContainerStateRequest {
        project_name,
        payload: states.clone(),
    };

    let response = client.post(url).json(&request).send().await?;
    response.error_for_status()?;
    Ok(())
}

pub(crate) async fn start(
    controller_url: &Url,
    project_name: ProjectName,
    hostname: HostName,
    client: reqwest::Client,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    info!("Starting monitor");
    let docker = Docker::connect_with_socket_defaults()?;
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        match fetch_container_info(&project_name, &docker).await {
            Ok(current_states) => {
                if let Err(e) = send_to_backend(
                    &client,
                    controller_url,
                    project_name.clone(),
                    hostname.clone(),
                    &current_states,
                )
                .await
                {
                    error!("Failed to send to backend: {e}");
                } else {
                    debug!(
                        "Successfully sent {} containers to backend",
                        current_states.len()
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
        assert!(log.contains("***REDACTED***"));
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
}
