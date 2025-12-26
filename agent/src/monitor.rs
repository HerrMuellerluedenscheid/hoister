use std::collections::HashMap;
use bollard::Docker;
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use log::{debug, error, info};
use std::time::Duration;
use bollard::query_parameters::ListContainersOptions;
use tokio::time;
use crate::docker::get_project_name;

async fn fetch_container_info(
    project_name: &str,
    docker: &Docker,
) -> Result<Vec<ContainerInspectResponse>, bollard::errors::Error> {

    let mut filters = HashMap::new();
    let label_filters = vec![
        format!("com.docker.compose.project={}", project_name),
    ];
    filters.insert("label".to_string(), label_filters);

    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };

    let containers = docker
        .list_containers(Some(options))
        .await?;

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

    let mut states = Vec::new();

    for container in containers {
        if let Some(id) = &container.id {
            match docker
                .inspect_container(
                    id,
                    None::<bollard::query_parameters::InspectContainerOptions>,
                )
                .await
            {
                Ok(inspect) => states.push(inspect),
                Err(e) => error!("Error inspecting container {}: {}", id, e),
            }
        }
    }

    Ok(states)
}


fn redact_credentials(inspect: &mut ContainerInspectResponse) {
    if let Some(config) = inspect.config.as_mut()
        && let Some(env_vars) = config.env.as_mut() {
            let sensitive_keywords = [
                "password", "passwd", "pwd",
                "secret", "token", "key",
                "auth", "credential", "cred",
                "apikey", "api_key",
                "username", "user",
                "session", "cookie",
            ];

            *env_vars = env_vars
                .iter()
                .map(|env_var| {
                    // Split on first '=' to get key=value
                    if let Some((key, _value)) = env_var.split_once('=') {
                        let key_lower = key.to_lowercase();

                        // Check if the key contains any sensitive keyword
                        let is_sensitive = sensitive_keywords
                            .iter()
                            .any(|keyword| key_lower.contains(keyword));

                        if is_sensitive {
                            format!("{}=***REDACTED***", key)
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
    controller_url: &str,
    states: &mut [ContainerInspectResponse],
) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!("{}/container/state", controller_url,);

    for state in states.iter_mut() {
        redact_credentials(state);
    }
    client
        .post(&url)
        .json(&serde_json::json!(states))
        .send()
        .await?;

    Ok(())
}

pub(crate) async fn start(
    controller_url: String,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    info!("Starting monitor");
    let docker = Docker::connect_with_socket_defaults()?;
    let mut interval = time::interval(Duration::from_secs(5));

    let project_name = get_project_name(&docker).await?;
    loop {
        interval.tick().await;

        match fetch_container_info(&project_name, &docker).await {
            Ok(mut current_states) => {
                if let Err(e) = send_to_backend(&controller_url, &mut current_states).await {
                    error!("Failed to send to backend: {}", e);
                } else {
                    debug!(
                        "Successfully sent {} containers to backend",
                        current_states.len()
                    );
                }
            }
            Err(e) => error!("Error fetching container info: {}", e),
        }
    }
}
