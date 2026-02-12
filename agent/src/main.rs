//! Fetch info of all running containers concurrently
mod config;
mod docker;
mod monitor;
mod notifications;
mod sse;

use bollard::Docker;

use bollard::query_parameters::EventsOptions;
use log::{debug, info};

#[cfg(target_os = "linux")]
use log::error;

use bollard::errors::Error as BollardError;

use crate::docker::{ContainerID, DockerHandler, get_project_name};
use bollard::models::ContainerCreateResponse;
use env_logger::Env;
use futures_util::StreamExt;
use std::collections::HashMap;
use std::default::Default;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::time::sleep;

use crate::notifications::{DeploymentResultHandler, setup_dispatcher, start_notification_handler};

use crate::sse::SSEHandler;
use hoister_shared::ProjectName;
use std::error::Error;
#[allow(unused_imports)]
use std::{env, process};
use tokio::sync::mpsc;

#[derive(Debug, Error)]
enum HoisterError {
    #[error("no update available")]
    NoUpdateAvailable,
    #[error("update failed: {0}")]
    UpdateFailed(String),
    #[error(transparent)]
    BollardError(#[from] BollardError),
    #[error("Docker failed: {0}")]
    Docker(String),
    #[error("Failed to get the project name")]
    ProjectNameDetectionFailed,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    #[cfg(target_os = "linux")]
    set_group_id();
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let config_path = "/hoister.toml";
    let config = Arc::new(config::load_config(config_path.as_ref()).await);
    let http_client = config::build_http_client(&config.controller);

    let (tx_notification, rx_notification) = mpsc::channel(32);
    let (tx_sse, rx_sse) = mpsc::channel(32);

    let result_handler = DeploymentResultHandler::new(tx_notification, config.hostname.clone());

    let _ = setup_dispatcher(&config).map(|d| {
        let c = Arc::clone(&config);
        let client = http_client.clone();
        tokio::spawn(async move {
            start_notification_handler(&c, rx_notification, d, client).await;
        });
    });

    info!("Starting hoister");
    if config.send_test_message {
        info!("Sending tests message");
        result_handler.test_message().await;
        // await 1 second to allow the message to be sent
        sleep(Duration::from_secs(1)).await;
        return Ok(());
    }
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
        info!("Received shutdown signal, gracefully shutting down...");
        process::exit(0);
    })
    .expect("Error setting Ctrl-C handler");

    let docker = Arc::new(DockerHandler::new(result_handler, config.registry.clone()));

    let mut sse_handler = SSEHandler::new(docker.clone(), rx_sse);
    tokio::spawn(async move {
        sse_handler.start().await;
    });

    let project_name = match &config.project {
        None => get_project_name(&docker.docker).await?,
        Some(pn) => pn.clone(),
    };

    if let Some(controller_config) = &config.controller {
        let mut url_sse = controller_config.url.clone();
        let url_state = controller_config.url.clone();
        url_sse.set_path("sse");
        let pn = project_name.clone();
        let hn = config.hostname.clone();
        let sse_client = http_client.clone();
        let monitor_client = http_client.clone();
        tokio::spawn(async move { sse::consume_sse(url_sse.as_str(), tx_sse, sse_client).await });
        tokio::spawn(async move {
            monitor::start(&url_state, pn, hn, monitor_client)
                .await
                .expect("Failed to start monitor");
        });
    }

    loop {
        debug!("---------- start checking containers ----------");
        run_update_check(&docker, &project_name).await?;
        let sleep = config.schedule.sleep();
        debug!("---------- end checking containers ----------");
        debug!("sleeping for {} seconds...", sleep.as_secs_f64());

        tokio::time::sleep(sleep).await;
    }
}

async fn run_update_check(
    docker: &DockerHandler,
    project_name: &ProjectName,
) -> Result<(), Box<dyn Error>> {
    let containers = docker.get_containers(project_name).await?;
    for container in containers {
        debug!("Checking container {:?}", container.id);
        let container_id: ContainerID = container.id.expect("container ID missing");
        let result = docker.update_container(project_name, &container_id).await;
        debug!("result: {result:?}");
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
