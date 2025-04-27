//! Fetch info of all running containers concurrently
mod cli;
mod docker;

use bollard::Docker;

use bollard::query_parameters::{EventsOptions, ListContainersOptions};
use log::{error, info};

use bollard::errors::Error as BollardError;

use crate::cli::configure_cli;
use crate::docker::update_container;
use bollard::models::ContainerCreateResponse;
use env_logger::Env;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
enum DeployaError {
    #[error("no update available")]
    NoUpdateAvailable,
    #[error(transparent)]
    BollardError(#[from] BollardError),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let docker = Docker::connect_with_local_defaults().unwrap();

    let config = configure_cli();
    let mut filters = HashMap::new();
    let label_filters = vec!["deploya.enable=true".to_string()];
    filters.insert("label".to_string(), label_filters);

    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };

    loop {
        let containers = docker
            .clone()
            .list_containers(Some(options.clone()))
            .await?;
        info!(
            "found {} containers with label `deploya.enable=true`",
            containers.len()
        );
        for container in containers {
            let _ = update_container(&docker, container)
                .await
                .inspect_err(|e| error!("{}", e));
            // monitor_state(container, &docker).await?;
        }
        if config.interval.is_some() {
            tokio::time::sleep(Duration::from_secs(config.interval.unwrap())).await;
        } else {
            break;
        }
    }

    Ok(())
}

async fn _monitor_state(
    container: ContainerCreateResponse,
    docker: &Docker,
) -> Result<(), DeployaError> {
    let container_to_monitor = container.id.clone();
    println!("Starting to monitor container: {}", container.id);

    let filters = HashMap::from([
        ("container".to_string(), vec![container_to_monitor.clone()]),
        ("type".to_string(), vec!["container".to_string()]),
    ]);
    let options = EventsOptions {
        since: None,
        until: None,
        filters: Some(filters),
    };

    let mut events_stream = docker.events(Some(options));

    while let Some(event) = events_stream.next().await {
        match event {
            Ok(event) => println!("event {:?}", event),
            Err(e) => eprintln!("Error receiving event: {:?}", e),
        }
    }

    Ok(())
}
