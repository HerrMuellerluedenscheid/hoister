use crate::HoisterError;
use crate::HoisterError::UpdateFailed;
use bollard::Docker;
use bollard::models::{
    ContainerCreateBody, ContainerCreateResponse, ContainerSummary, HealthStatusEnum,
};
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, InspectContainerOptions, RemoveContainerOptions,
    RenameContainerOptions, StartContainerOptions, StopContainerOptionsBuilder,
};
use futures_util::StreamExt;
use log::{debug, error, info, warn};
use std::time::Duration;

pub(crate) async fn update_container(
    docker: &Docker,
    container: ContainerSummary,
) -> Result<ContainerCreateResponse, HoisterError> {
    let container_id = container.id.unwrap_or_default();
    let container_details = docker
        .inspect_container(&container_id, None::<InspectContainerOptions>)
        .await?;
    let old_config = container_details.clone().config.unwrap();
    let image_name = old_config.image.unwrap();

    let (image_name, image_tag) = image_name.rsplit_once(":").unwrap_or_default();
    let image_name = image_name.to_string();
    let image_tag = image_tag.to_string();

    debug!(
        "container details: {}",
        serde_json::to_string_pretty(&container_details).unwrap()
    );

    info!("Pulling update for: {}:{}", image_name, image_tag);

    let digest = download_image(docker, &image_name, &image_tag).await?;
    debug!("Image pulled successfully (digest: {})", digest);
    let new_image_name = image_name.clone() + "@" + &digest;
    debug!("new image name: {}", new_image_name);
    info!("Stopping container {:?}...", &container_id);
    let options_stop_container = StopContainerOptionsBuilder::new().t(30).build();

    docker
        .stop_container(&container_id, Some(options_stop_container.clone()))
        .await?;

    let backup_name = format!("{}-backup", container_id);
    debug!("rename old container to {}", &backup_name);

    let rename_options = RenameContainerOptions {
        name: backup_name.clone(),
    };
    docker
        .rename_container(&container_id, rename_options)
        .await?;

    let host_config = container_details.host_config.unwrap_or_default();

    let name = container_details
        .name
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let mut config = ContainerCreateBody {
        host_config: Some(host_config),
        ..Default::default()
    };

    if let Some(old_config) = container_details.config {
        config.env = old_config.env;
        // config.cmd = old_config.cmd;
        // config.entrypoint = old_config.entrypoint;
        config.labels = old_config.labels;
        config.exposed_ports = old_config.exposed_ports;
        config.image = old_config.image;
        config.attach_stderr = old_config.attach_stderr;
        config.attach_stdout = old_config.attach_stdout;
        config.tty = old_config.tty;
    }

    let options = CreateContainerOptions {
        name: Some(name),
        ..Default::default()
    };

    let container = docker.create_container(Some(options), config).await?;
    debug!("Container created with ID: {}", container.id);

    docker
        .start_container(&container.id, None::<StartContainerOptions>)
        .await?;
    info!("Container started");

    if let Err(e) = check_container_health(docker, &container.id).await {
        warn!("New container failed, rolling back to previous version");

        docker
            .stop_container(&container.id, Some(options_stop_container))
            .await?;
        docker
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
        docker
            .rename_container(&backup_name, rename_back_options)
            .await?;

        docker
            .start_container(&container_id, None::<StartContainerOptions>)
            .await?;
        info!("Rollback complete, old container restarted");
        return Err(e);
    } else {
        debug!("Container updated successfully. deleting old container");
        let remove_options = RemoveContainerOptions {
            v: false,
            force: false,
            link: false,
        };
        docker
            .remove_container(&container_id, Some(remove_options))
            .await?;
        info!("Container updated successfully. backup container removed");
    }
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
                debug!("{:?}", output);
                if let Some(status) = &output.status {
                    if status.contains("Download complete") || status.contains("Pull complete") {
                        update_available = true;
                    }
                    if status.contains("Digest:") {
                        if let Some(pos) = status.find("sha256:") {
                            status[pos..].clone_into(&mut digest);
                        }
                    }
                }
            }
            Err(e) => error!("Error pulling image: {:?}", e),
        }
    }
    if !update_available {
        return Err(HoisterError::NoUpdateAvailable);
    }
    Ok(digest)
}

async fn check_container_health(docker: &Docker, container_name: &str) -> Result<(), HoisterError> {
    tokio::time::sleep(Duration::from_secs(5)).await;

    let container = docker
        .inspect_container(container_name, None::<InspectContainerOptions>)
        .await?;

    if let Some(state) = container.state {
        if let Some(running) = state.running {
            if running {
                if let Some(health) = state.health {
                    if let Some(status) = health.status {
                        if status == HealthStatusEnum::HEALTHY {
                            return Ok(());
                        }
                    }
                } else {
                    return Ok(());
                }
            }
        }
    }

    Err(UpdateFailed(container.config.unwrap().image.unwrap()))
}
