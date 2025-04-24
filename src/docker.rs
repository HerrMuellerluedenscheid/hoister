use bollard::Docker;
use bollard::models::{ContainerCreateBody, ContainerSummary};
use bollard::query_parameters::{CreateContainerOptions, CreateImageOptions, InspectContainerOptions, RemoveContainerOptions, StartContainerOptions, StopContainerOptionsBuilder};
use futures_util::StreamExt;
use log::{debug, error, info};
use crate::DeployaError;

pub(crate) async fn update_container(
    docker: &Docker,
    container: ContainerSummary,
) -> Result<(), DeployaError> {
    let container_id = container.id.unwrap_or_default();
    let image_name = container.image.expect("Container tag format wrong");
    debug!("Image: {}", image_name);
    let (image_name, image_tag) = image_name.rsplit_once(":").unwrap_or_default();
    let image_name = image_name.to_string();
    let image_tag = image_tag.to_string();
    let container_details = docker
        .inspect_container(&container_id, None::<InspectContainerOptions>)
        .await?;
    debug!(
        "container details: {}",
        serde_json::to_string_pretty(&container_details).unwrap()
    );
    debug!("ID: {}", &container_id);
    debug!("Names: {:?}", container.names.unwrap_or_default());
    debug!("Image: {}", image_name);
    let image_id = container.image_id.unwrap_or_default();
    let image_details = docker.inspect_image(&image_id).await?;
    debug!(
        "Image Details: {}",
        serde_json::to_string_pretty(&image_details).unwrap()
    );

    info!(
        "Pulling latest version of the image...{}:{}",
        image_name, image_tag
    );

    let digest = download_image(docker, &image_name, image_tag).await?;
    debug!("Image pulled successfully (digest: {})", digest);
    let new_image_name = image_name + "@" + &digest;
    info!("new image name: {}", new_image_name);
    let image_details = docker.inspect_image(&new_image_name).await?;
    info!(
        "Image Details of new image: {}",
        serde_json::to_string_pretty(&image_details).unwrap()
    );
    info!("Stopping container {:?}...", &container_id);
    let options = StopContainerOptionsBuilder::new().t(30).build();

    docker.stop_container(&container_id, Some(options)).await?;

    let remove_options = RemoveContainerOptions {
        v: false,     // Don't remove volumes
        force: false, // Container is already stopped
        link: false,
    };
    docker
        .remove_container(&container_id, Some(remove_options))
        .await?;

    let host_config = container_details.host_config.unwrap_or_default();

    let name = container_details
        .name
        .unwrap_or_default()
        .trim_start_matches('/')
        .to_string();

    let mut config: ContainerCreateBody = ContainerCreateBody::default();
    config.host_config = Some(host_config);

    if let Some(old_config) = container_details.config {
        config.env = old_config.env;
        config.cmd = old_config.cmd;
        config.entrypoint = old_config.entrypoint;
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
    info!("Container created with ID: {}", container.id);

    docker
        .start_container(&container.id, None::<StartContainerOptions>)
        .await?;
    info!("Container started successfully");
    Ok(())
}

async fn download_image(
    docker: &Docker,
    image_name: &String,
    image_tag: String,
) -> Result<String, DeployaError> {
    let mut update_available = false;
    let mut digest = String::new();
    let options = CreateImageOptions {
        from_image: Some(image_name.clone()),
        tag: Some(image_tag.to_string()),
        ..Default::default()
    };
    let mut pull_stream = docker.create_image(Some(options), None, None);
    while let Some(result) = pull_stream.next().await {
        match result {
            Ok(output) => {
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
        return Err(DeployaError::NoUpdateAvailable);
    }
    Ok(digest)
}
