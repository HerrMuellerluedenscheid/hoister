//! Fetch info of all running containers concurrently
mod cli;
mod docker;
mod notifications;

use bollard::Docker;

use bollard::query_parameters::{EventsOptions, ListContainersOptions};
use log::{debug, error, info};

use bollard::errors::Error as BollardError;

use crate::cli::configure_cli;
use crate::docker::update_container;
use bollard::models::{ContainerCreateResponse, ContainerSummary};
use env_logger::Env;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::default::Default;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::time::sleep;

use crate::notifications::{setup_dispatcher, start_notification_handler};
use chatterbox::message::Message;
use controller::server::{CreateDeployment, DeploymentStatus};
use std::error::Error;
#[allow(unused_imports)]
use std::{env, process};
use tokio::sync::mpsc;

#[derive(Debug)]
struct DeploymentResult {
    image: String,
    status: DeploymentStatus,
}

impl From<&DeploymentResult> for Message {
    fn from(val: &DeploymentResult) -> Self {
        Message::new(val.status.to_string(), val.image.clone())
    }
}

impl From<&DeploymentResult> for CreateDeployment {
    fn from(result: &DeploymentResult) -> Self {
        CreateDeployment {
            image: result.image.clone(),
            status: result.status.clone(),
        }
    }
}

#[derive(Debug, Error)]
enum HoisterError {
    #[error("no update available")]
    NoUpdateAvailable,
    #[error("update failed: {0}")]
    UpdateFailed(String),
    #[error(transparent)]
    BollardError(#[from] BollardError),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    #[cfg(target_os = "linux")]
    set_group_id();

    let (tx, rx) = mpsc::channel(32);

    let dispatcher = setup_dispatcher();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    tokio::spawn(async move {
        start_notification_handler(rx, dispatcher).await;
    });
    info!("Starting hoister");
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        info!("Received shutdown signal, gracefully shutting down...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let docker = Docker::connect_with_local_defaults().unwrap();

    let config = configure_cli();

    loop {
        info!("checking for updates");
        let now = SystemTime::now();
        let containers = get_containers(&docker).await?;
        for container in containers {
            debug!("Checking container {:?}", container);
            let image = container.clone().image.unwrap_or_default();
            let result = match update_container(&docker, container).await {
                Ok(_response) => DeploymentResult {
                    image: image.clone(),
                    status: DeploymentStatus::Success,
                },
                Err(HoisterError::NoUpdateAvailable) => DeploymentResult {
                    image: image.clone(),
                    status: DeploymentStatus::NoUpdate,
                },
                Err(e) => {
                    error!("failed to update container: {}", e);
                    DeploymentResult {
                        image: image.clone(),
                        status: DeploymentStatus::Failure,
                    }
                }
            };
            tx.send(result).await.unwrap();
        }

        if config.interval.is_some() {
            while running.load(Ordering::SeqCst)
                && now.elapsed().unwrap() < Duration::from_secs(config.interval.unwrap())
            {
                sleep(Duration::from_millis(500)).await;
            }
            if !running.load(Ordering::SeqCst) {
                break;
            }
        } else {
            break;
        }
    }
    Ok(())
}

async fn get_containers(docker: &Docker) -> Result<Vec<ContainerSummary>, Box<dyn Error>> {
    let mut filters = HashMap::new();
    let label_filters = vec!["hoister.enable=true".to_string()];
    filters.insert("label".to_string(), label_filters);

    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };
    let containers = docker
        .clone()
        .list_containers(Some(options.clone()))
        .await?;

    debug!(
        "found {} containers with label `hoister.enable=true`",
        containers.len()
    );
    Ok(containers)
}

#[cfg(target_os = "linux")]
fn set_group_id() {
    let docker_gid = env::var("DOCKER_GID")
        .unwrap_or_else(|_| "999".to_string())
        .parse::<u32>()
        .expect("Invalid DOCKER_GID");
    info!("Setting GID to {}", docker_gid);
    // Note: This requires CAP_SETGID capability
    unsafe {
        if libc::setgid(docker_gid) != 0 {
            error!("Failed to set GID to {}", docker_gid);
        }
    }
}

async fn _monitor_state(
    container: ContainerCreateResponse,
    docker: &Docker,
) -> Result<(), HoisterError> {
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
            Ok(event) => println!("event {event:?}"),
            Err(e) => eprintln!("Error receiving event: {e:?}"),
        }
    }

    Ok(())
}
