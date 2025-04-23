//! Fetch info of all running containers concurrently

use bollard::Docker;

use bollard::models::ContainerCreateBody;
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, InspectContainerOptions, ListContainersOptions,
    RemoveContainerOptions, StartContainerOptions, StopContainerOptions,
    StopContainerOptionsBuilder,
};

use bollard::secret::ContainerSummaryStateEnum;
use bollard::service::HostConfig;

use futures_util::stream::StreamExt;
use std::collections::HashMap;
use std::default::Default;
use std::process::exit;
use std::time::Duration;

use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + 'static>> {
    let docker = Docker::connect_with_local_defaults().unwrap();

    // Create a filter for containers with the label "deploy.enable=true"
    let mut filters = HashMap::new();
    let mut label_filters = Vec::new();
    label_filters.push("deploya.enable=true".to_string());
    filters.insert("label".to_string(), label_filters);

    // Set up the list options with our filter
    let options = ListContainersOptions {
        all: true, // Include stopped containers as well
        filters: Some(filters),
        ..Default::default()
    };

    // List the containers matching our filter
    let containers = docker.clone().list_containers(Some(options)).await?;
    // Print the container details
    println!("found {} containers", containers.len());
    for container in containers {
        let container_id = container.id.unwrap_or_default();
        let container_state = container.state.unwrap();
        match container_state {
            ContainerSummaryStateEnum::RUNNING | ContainerSummaryStateEnum::RESTARTING => {
                println!("Container {} is {}", container_id, &container_state);
            }
            _ => {
                println!("Container {} is {}", container_id, &container_state);
                continue;
            }
        }
        let image_name = container.image.unwrap_or("x:x".into());
        println!("Image: {}", image_name);
        let (image_name, image_tag) = image_name.rsplit_once(":").unwrap_or_default();
        let image_name = image_name.to_string();
        let image_tag = image_tag.to_string();
        let container_details = docker
            .inspect_container(&container_id, None::<InspectContainerOptions>)
            .await?;
        println!("container details: {:?}", container_details);
        println!("ID: {}", &container_id);
        println!("Names: {:?}", container.names.unwrap_or_default());
        println!("Image: {}", image_name);
        let image_id = container.image_id.unwrap_or_default();
        let image_details = docker.inspect_image(&image_id).await.unwrap();
        println!("Image Details: {:?}", image_details);
        if let Some(repo_tags) = image_details.repo_tags {
            println!("Full image URL: {:?}", repo_tags);
        }

        // Step 3: Pull the latest version of the image
        println!(
            "Pulling latest version of the image...{}:{}",
            image_name, image_tag
        );
        let options = CreateImageOptions {
            from_image: Some(image_name.clone()),
            tag: Some(image_tag.to_string()), // Or specify a different tag if needed
            ..Default::default()
        };
        let mut update_available = false;
        let mut pull_stream = docker.create_image(Some(options), None, None);
        while let Some(result) = pull_stream.next().await {
            match result {
                Ok(output) => {
                    if let Some(status) = &output.status {
                        if status.contains("Download complete") || status.contains("Pull complete")
                        {
                            update_available = true;
                        }
                        println!("Status: {}", status);
                    }
                    println!("{:?}", output)
                }
                Err(e) => eprintln!("Error pulling image: {:?}", e),
            }
        }
        println!("Image pulled successfully");
        if !update_available {
            println!("No update available");
            exit(0);
        }

        println!("Stopping container {:?}...", &container_id);
        let options = StopContainerOptionsBuilder::new().t(30).build();

        docker.stop_container(&container_id, Some(options)).await?;
        println!("Container stopped successfully");

        println!("Removing container (keeping volumes)...");
        let remove_options = RemoveContainerOptions {
            v: false,     // Don't remove volumes
            force: false, // Container is already stopped
            link: false,
        };
        docker
            .remove_container(&container_id, Some(remove_options))
            .await?;
        println!("Container removed successfully");
        println!("Creating new container with updated image...");

        // Extract necessary configuration from the container details
        let host_config = container_details.host_config.unwrap_or_default();
        let network_mode = host_config.network_mode.unwrap_or_default();
        let binds = host_config.binds.unwrap_or_default();

        let name = container_details
            .name
            .unwrap_or_default()
            .trim_start_matches('/')
            .to_string();

        let mut host_config = HostConfig::default();
        host_config.network_mode = Some(network_mode);
        host_config.binds = Some(binds);

        // Add any other host config parameters needed (mounts, ports, etc.)
        if let Some(mounts) = host_config.mounts {
            host_config.mounts = Some(mounts);
        }
        if let Some(port_bindings) = host_config.port_bindings {
            println!("setting port binding: {:?}", port_bindings);
            host_config.port_bindings = Some(port_bindings);
        }

        let mut config: ContainerCreateBody = ContainerCreateBody::default();
        // let mut config = Config::default();
        // config.image = Some(image_name);
        config.host_config = Some(host_config);

        // Copy over environment variables, entrypoint, cmd, etc.
        if let Some(old_config) = container_details.config {
            config.env = old_config.env;
            config.cmd = old_config.cmd;
            config.entrypoint = old_config.entrypoint;
            config.labels = old_config.labels;
            config.exposed_ports = old_config.exposed_ports;
            config.image = old_config.image;
            // Add any other config parameters needed
        }

        let options = CreateContainerOptions {
            name: Some(name),
            // Add any other options needed
            ..Default::default()
        };

        let container = docker.create_container(Some(options), config).await?;
        println!("Container created with ID: {}", container.id);

        // Step 6: Start the new container
        println!("Starting new container...");
        docker
            .start_container(&container.id, None::<StartContainerOptions>)
            .await?;
        println!("Container started successfully");

        // Do something while the container is running
        println!("Container is running...");
        println!("-----------------------");
    }

    Ok(())
}
