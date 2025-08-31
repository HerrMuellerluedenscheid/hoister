//! Fetch info of all running containers concurrently
mod cli;
mod docker;
mod notifications;

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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use tokio::time::sleep;

use crate::notifications::setup_dispatcher;
use chatterbox::message::Message;
#[allow(unused_imports)]
use std::{env, process};

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

    let dispatcher = setup_dispatcher();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

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
    let mut filters = HashMap::new();
    let label_filters = vec!["hoister.enable=true".to_string()];
    filters.insert("label".to_string(), label_filters);

    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };

    let mut n_containers = 0;
    loop {
        let now = SystemTime::now();
        let containers = docker
            .clone()
            .list_containers(Some(options.clone()))
            .await?;
        if n_containers != containers.len() {
            info!(
                "found {} containers with label `hoister.enable=true`",
                containers.len()
            );
            n_containers = containers.len();
        }

        for container in containers {
            let image = container.clone().image.unwrap_or_default();
            let message = match update_container(&docker, container).await {
                Ok(_) => {
                    info!("deployed updated image {}", image);
                    Some(Message::new(
                        "update successfully deployed".to_string(),
                        format!("image {} deployed", image),
                    ))
                }
                Err(e) => e.into(),
            };

            if let Some(message) = message {
                _ = dispatcher
                    .dispatch(&message)
                    .await
                    .inspect_err(|e| error!("failed to dispatch message: {}", e));
            }
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
            Ok(event) => println!("event {:?}", event),
            Err(e) => eprintln!("Error receiving event: {:?}", e),
        }
    }

    Ok(())
}
