//! Fetch info of all running containers concurrently

mod docker;

use bollard::Docker;
use serde_json;

use bollard::models::{ContainerCreateBody, ContainerSummary};
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, InspectContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptionsBuilder,
};
use env_logger;
use log::{debug, error, info};

use bollard::errors::Error as BollardError;
use bollard::secret::ContainerSummaryStateEnum;

use env_logger::Env;
use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use thiserror::Error;
use crate::docker::update_container;

#[derive(Debug, Error)]
enum DeployaError {
    #[error("no update available")]
    NoUpdateAvailable,
    #[error("container {0} not running")]
    ContainerNotRunning(String),
    #[error(transparent)]
    BollardError(#[from] BollardError),
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let docker = Docker::connect_with_local_defaults().unwrap();

    let mut filters = HashMap::new();
    let mut label_filters = Vec::new();
    label_filters.push("deploya.enable=true".to_string());
    filters.insert("label".to_string(), label_filters);

    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };

    let containers = docker.clone().list_containers(Some(options)).await?;
    info!(
        "found {} containers with label `deploya.enable=true`",
        containers.len()
    );
    for container in containers {
        let _ = update_container(&docker, container)
            .await
            .inspect_err(|e| error!("{}", e));
    }

    Ok(())
}
