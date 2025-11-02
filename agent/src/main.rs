//! Fetch info of all running containers concurrently
mod cli;
mod docker;
mod notifications;
mod sse;

use bollard::Docker;

use bollard::query_parameters::EventsOptions;
use log::{debug, error, info};

use bollard::errors::Error as BollardError;

use crate::cli::configure_cli;
use crate::docker::{ContainerID, DockerHandler};
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

use crate::notifications::{DeploymentResultHandler, setup_dispatcher, start_notification_handler};
use chatterbox::message::Message;
use controller::server::{CreateDeployment, DeploymentStatus};
use controller::sse::ControllerEvent;
#[allow(unused_imports)]
use std::{env, process};
use tokio::sync::mpsc;

#[derive(Debug)]
struct DeploymentResult {
    image: String,
    container_id: ContainerID,
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
            container_id: result.container_id.clone(),
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

struct SSEHandler {
    docker: Arc<DockerHandler>,
    rx: mpsc::Receiver<ControllerEvent>,
}

impl SSEHandler {
    fn new(docker: Arc<DockerHandler>, rx: mpsc::Receiver<ControllerEvent>) -> Self {
        Self { docker, rx }
    }

    async fn start(&mut self) {
        while let Some(message) = self.rx.recv().await {
            match message {
                ControllerEvent::Retry(container_id) => {
                    self.docker
                        .update_container(&container_id)
                        .await
                        .expect("TODO: panic message");
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    #[cfg(target_os = "linux")]
    set_group_id();

    let (tx_notification, rx_notification) = mpsc::channel(32);
    let (tx_sse, rx_sse) = mpsc::channel(32);

    let result_handler = DeploymentResultHandler::new(tx_notification);

    if let Ok(controller_url) = env::var("HOISTER_CONTROLLER_URL") {
        tokio::spawn(async move {
            sse::consume_sse(format!("{controller_url}/sse").as_str(), tx_sse).await
        });
    }
    let dispatcher = setup_dispatcher();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    tokio::spawn(async move {
        start_notification_handler(rx_notification, dispatcher).await;
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

    let docker = Arc::new(DockerHandler::new(result_handler));

    let mut sse_handler = SSEHandler::new(docker.clone(), rx_sse);
    tokio::spawn(async move {
        sse_handler.start().await;
    });
    let config = configure_cli();

    loop {
        info!("checking for updates");
        let now = SystemTime::now();
        let containers = docker.get_containers().await?;
        for container in containers {
            debug!("Checking container {:?}", container.id);
            let container_id: ContainerID = container.id.unwrap_or_default();
            let result = docker.update_container(&container_id).await;
            info!("result: {:?}", result);
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
            Ok(event) => println!("event {event:?}"),
            Err(e) => eprintln!("Error receiving event: {e:?}"),
        }
    }

    Ok(())
}
