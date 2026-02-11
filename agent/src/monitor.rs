use crate::HoisterError;
use crate::docker::get_service_identifier;
use bollard::Docker;
use bollard::models::{ContainerInspectResponse, ContainerSummary};
use bollard::query_parameters::ListContainersOptions;
use controller::inbound::server::PostContainerStateRequest;
use hoister_shared::{HostName, ProjectName, ServiceName};
use log::{debug, error, info};
use reqwest::Url;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

async fn fetch_container_info(
    #[allow(unused_variables)] project_name: &ProjectName,
    docker: &Docker,
) -> Result<HashMap<ServiceName, ContainerInspectResponse>, HoisterError> {
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
                    redact_credentials(&mut inspect);
                    states.insert(service_identifier.clone(), inspect);
                }
                Err(e) => error!("Error inspecting container {}: {}", container_id, e),
            }
        }
    }

    Ok(states)
}

fn redact_credentials(inspect: &mut ContainerInspectResponse) {
    if let Some(config) = inspect.config.as_mut()
        && let Some(env_vars) = config.env.as_mut()
    {
        let sensitive_keywords = [
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
    controller_url: &Url,
    project_name: ProjectName,
    hostname: HostName,
    states: &HashMap<ServiceName, ContainerInspectResponse>,
) -> Result<(), reqwest::Error> {
    let url = controller_url
        .join(format!("container/state/{}/{}", hostname.0, project_name.0).as_str())
        .expect("failed to join url");

    let request = PostContainerStateRequest {
        project_name,
        payload: states.clone(),
    };

    let client = reqwest::Client::new();
    let response = client.post(url).json(&request).send().await?;
    response.error_for_status()?;
    Ok(())
}

pub(crate) async fn start(
    controller_url: &Url,
    project_name: ProjectName,
    hostname: HostName,
) -> Result<(), Box<dyn std::error::Error + 'static>> {
    info!("Starting monitor");
    let docker = Docker::connect_with_socket_defaults()?;
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        match fetch_container_info(&project_name, &docker).await {
            Ok(current_states) => {
                if let Err(e) = send_to_backend(
                    controller_url,
                    project_name.clone(),
                    hostname.clone(),
                    &current_states,
                )
                .await
                {
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
