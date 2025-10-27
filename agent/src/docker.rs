use crate::HoisterError;
use crate::HoisterError::UpdateFailed;
use bollard::Docker;
use bollard::models::{
    ContainerCreateBody, ContainerCreateResponse, ContainerInspectResponse, ContainerSummary,
    HealthStatusEnum,
};
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, InspectContainerOptions, ListContainersOptions,
    RemoveContainerOptions, RenameContainerOptions, StartContainerOptions,
    StopContainerOptionsBuilder,
};
use futures_util::StreamExt;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

pub(crate) type ContainerID = String;
pub(crate) type ContainerIdentifier = String; // used to identify the image across hoister services

const REMOVE_OPTIONS: RemoveContainerOptions = RemoveContainerOptions {
    v: false,
    force: false,
    link: false,
};

pub(crate) struct DockerHandler {
    docker: Docker,
}

impl DockerHandler {
    pub(crate) fn new() -> Self {
        let docker = Docker::connect_with_local_defaults().unwrap();
        Self { docker }
    }

    pub(crate) async fn get_image_identifier(
        &self,
        container_id: &ContainerID,
    ) -> Result<ContainerIdentifier, HoisterError> {
        let container_details = self
            .docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await
            .inspect_err(|x| error!("Error inspecting container: {x:?}"))?;

        // if hoister.identifier is set use that as the identifier
        let identifier = container_details
            .clone()
            .config
            .unwrap()
            .labels
            .unwrap_or_default()
            .get("hoister.identifier")
            .cloned();
        if let Some(id) = identifier {
            return Ok(id.to_string());
        }

        let image_inspect = self
            .docker
            .inspect_image(&container_details.clone().config.unwrap().image.unwrap())
            .await
            .inspect_err(|x| error!("Error inspecting image: {x:?}"))?;

        let repo_digests = image_inspect.repo_digests.unwrap_or(vec![
            container_details.name.unwrap_or("unknown".to_string()),
        ]);
        Ok(repo_digests.first().unwrap().to_string())
    }

    pub(crate) async fn update_container(
        &self,
        container_id: &ContainerID,
    ) -> Result<ContainerCreateResponse, HoisterError> {
        let container_details = self
            .docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await?;

        let old_config = container_details.clone().config.unwrap();
        let image_name = old_config.image.unwrap();

        let (image_name, image_tag) = image_name.rsplit_once(":").unwrap_or_default();
        let image_name = image_name.to_string();
        let image_tag = image_tag.to_string();

        trace!(
            "container details: {}",
            serde_json::to_string_pretty(&container_details).unwrap()
        );

        info!("Checking for updates: {image_name}:{image_tag}");

        let digest = download_image(&self.docker, &image_name, &image_tag).await?;
        debug!("Image pulled successfully (digest: {digest})");
        let new_image_name = image_name.clone() + "@" + &digest;
        debug!("new image name: {new_image_name}");
        info!("Stopping container {:?}...", &container_id);
        let options_stop_container = StopContainerOptionsBuilder::new().t(30).build();
        self.docker
            .stop_container(container_id, Some(options_stop_container.clone()))
            .await?;

        let backup_name = format!("{container_id}-backup");
        debug!("rename old container to {}", &backup_name);

        let rename_options = RenameContainerOptions {
            name: backup_name.clone(),
        };
        self.docker
            .rename_container(container_id, rename_options)
            .await?;

        let container = create_container(&self.docker, container_details).await?;
        debug!("Container created with ID: {}", container.id);

        self.docker
            .start_container(&container.id, None::<StartContainerOptions>)
            .await?;
        info!("Container started");

        if let Err(e) = check_container_health(&self.docker, &container.id).await {
            warn!("New container failed, rolling back to previous version");

            self.docker
                .stop_container(&container.id, Some(options_stop_container))
                .await?;
            self.docker
                .remove_container(
                    &container.id,
                    Some(RemoveContainerOptions {
                        force: true,
                        ..Default::default()
                    }),
                )
                .await?;

            let rename_back_options = RenameContainerOptions {
                name: container_id.clone(),
            };
            self.docker
                .rename_container(&backup_name, rename_back_options)
                .await?;

            self.docker
                .start_container(container_id, None::<StartContainerOptions>)
                .await?;
            info!("Rollback complete, old container restarted");
            return Err(e);
        } else {
            debug!("Container updated successfully. deleting old container");
            self.docker
                .remove_container(container_id, Some(REMOVE_OPTIONS))
                .await?;
            info!("Container updated successfully. backup container removed");
        }
        Ok(container)
    }

    pub(crate) async fn get_containers(&self) -> Result<Vec<ContainerSummary>, Box<dyn Error>> {
        let mut filters = HashMap::new();
        let label_filters = vec!["hoister.enable=true".to_string()];
        filters.insert("label".to_string(), label_filters);

        let options = ListContainersOptions {
            filters: Some(filters),
            ..Default::default()
        };
        let containers = self
            .docker
            .clone()
            .list_containers(Some(options.clone()))
            .await?;

        debug!(
            "found {} containers with label `hoister.enable=true`",
            containers.len()
        );
        Ok(containers)
    }
}

async fn create_container(
    docker: &Docker,
    container_details: ContainerInspectResponse,
) -> Result<ContainerCreateResponse, HoisterError> {
    let host_config = container_details.host_config.unwrap_or_default();

    let mut config = ContainerCreateBody {
        host_config: Some(host_config),
        ..Default::default()
    };

    if let Some(last_config) = container_details.config {
        config.env = last_config.env;
        // config.cmd = last_config.cmd;
        // config.entrypoint = last_config.entrypoint;
        config.labels = last_config.labels;
        config.exposed_ports = last_config.exposed_ports;
        config.image = last_config.image;
        config.attach_stderr = last_config.attach_stderr;
        config.attach_stdout = last_config.attach_stdout;
        config.tty = last_config.tty;
    }

    let name = container_details
        .name
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let options = CreateContainerOptions {
        name: Some(name),
        ..Default::default()
    };

    let container = docker.create_container(Some(options), config).await?;
    Ok(container)
}

async fn download_image(
    docker: &Docker,
    image_name: &str,
    image_tag: &str,
) -> Result<String, HoisterError> {
    let mut update_available = false;
    let mut digest = String::new();
    let options = CreateImageOptions {
        from_image: Some(image_name.to_owned()),
        tag: Some(image_tag.to_owned()),
        ..Default::default()
    };
    let mut pull_stream = docker.create_image(Some(options), None, None);
    while let Some(result) = pull_stream.next().await {
        match result {
            Ok(output) => {
                debug!("{output:?}");
                if let Some(status) = &output.status {
                    if status.contains("Download complete")
                        || status.contains("Pull complete")
                        || status.contains("Downloaded newer image for")
                    {
                        update_available = true;
                    }
                    if status.contains("Digest:")
                        && let Some(pos) = status.find("sha256:")
                    {
                        status[pos..].clone_into(&mut digest);
                    }
                }
            }
            Err(e) => error!("Error pulling image: {e:?}"),
        }
    }
    if !update_available {
        return Err(HoisterError::NoUpdateAvailable);
    }
    info!("New image pulled");
    Ok(digest)
}

async fn check_container_health(docker: &Docker, container_name: &str) -> Result<(), HoisterError> {
    tokio::time::sleep(Duration::from_secs(5)).await;

    let container = docker
        .inspect_container(container_name, None::<InspectContainerOptions>)
        .await?;

    if let Some(state) = container.state
        && let Some(running) = state.running
        && running
    {
        if let Some(health) = state.health {
            if let Some(status) = health.status
                && status == HealthStatusEnum::HEALTHY
            {
                return Ok(());
            }
        } else {
            return Ok(());
        }
    }

    Err(UpdateFailed(container.config.unwrap().image.unwrap()))
}
