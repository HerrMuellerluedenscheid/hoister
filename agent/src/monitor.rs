use bollard::Docker;
use bollard::models::ContainerInspectResponse;
use env_logger::Env;
use log::info;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::time::Duration;
use tokio::time;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct ContainerState {
    id: String,
    name: String,
    status: String,
    image: String,
}

impl From<&ContainerInspectResponse> for ContainerState {
    fn from(inspect: &ContainerInspectResponse) -> Self {
        Self {
            id: inspect.id.clone().unwrap_or_default(),
            name: inspect.name.clone().unwrap_or_default(),
            status: inspect
                .state
                .as_ref()
                .and_then(|s| s.status.as_ref())
                .map(|s| s.to_string())
                .unwrap_or_default(),
            image: inspect.image.clone().unwrap_or_default(),
        }
    }
}

async fn fetch_container_info(
    docker: &Docker,
) -> Result<Vec<ContainerInspectResponse>, bollard::errors::Error> {
    let containers = docker
        .list_containers(Some(
            bollard::query_parameters::ListContainersOptionsBuilder::default()
                .all(true)
                .build(),
        ))
        .await?;

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
                Err(e) => eprintln!("Error inspecting container {}: {}", id, e),
            }
        }
    }

    Ok(states)
}

async fn send_to_backend(controller_url: &str, states: &[ContainerInspectResponse]) -> Result<(), reqwest::Error> {
    let client = reqwest::Client::new();
    let url = format!(
        "{}/container/state",
        controller_url,
    );
    client
        .post(&url)
        .json(&serde_json::json!(states))
        .send()
        .await?;
    info!("done Sending to backend: {}", url);

    Ok(())
}

pub(crate) async fn start(controller_url: String) -> Result<(), Box<dyn std::error::Error + 'static>> {
    info!("Starting monitor");
    let docker = Docker::connect_with_socket_defaults()?;
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;

        match fetch_container_info(&docker).await {
            Ok(current_states) => {
                if let Err(e) = send_to_backend(&controller_url, &current_states).await {
                    eprintln!("Failed to send to backend: {}", e);
                } else {
                    println!(
                        "Successfully sent {} containers to backend",
                        current_states.len()
                    );
                }
            }
            Err(e) => eprintln!("Error fetching container info: {}", e),
        }
    }
}
