use crate::HoisterError;
use crate::HoisterError::UpdateFailed;
use crate::env;
use crate::notifications::DeploymentResultHandler;
use bollard::Docker;
use bollard::auth::DockerCredentials;
use bollard::models::{
    ContainerCreateBody, ContainerCreateResponse, ContainerInspectResponse, ContainerSummary,
    HealthStatusEnum, MountPointTypeEnum, VolumeCreateOptions,
};
use bollard::query_parameters::{
    CreateContainerOptions, CreateImageOptions, InspectContainerOptions, ListContainersOptions,
    RemoveContainerOptions, RemoveVolumeOptions, RenameContainerOptions, StartContainerOptions,
    StopContainerOptionsBuilder, WaitContainerOptions, WaitContainerOptionsBuilder,
};
use futures_util::{StreamExt, TryStreamExt};
use lazy_static::lazy_static;
use log::{debug, error, info, trace, warn};
use std::collections::HashMap;
use std::env::current_dir;
use std::error::Error;
use std::path::Path;
use std::time::Duration;

lazy_static! {
    static ref CREDENTIALS: DockerCredentials = DockerCredentials {
        username: env::var("HOISTER_REGISTRY_USERNAME").ok(),
        password: env::var("HOISTER_REGISTRY_PASSWORD").ok(),
        auth: env::var("HOISTER_REGISTRY_AUTH").ok(),
        email: env::var("HOISTER_REGISTRY_EMAIL").ok(),
        serveraddress: env::var("HOISTER_REGISTRY_SERVERADDRESS").ok(),
        identitytoken: env::var("HOISTER_REGISTRY_IDENTITYTOKEN").ok(),
        registrytoken: env::var("HOISTER_REGISTRY_REGISTRYTOKEN").ok(),
    };
}

pub(crate) type ContainerID = String;
pub(crate) type ContainerIdentifier = String;
pub(crate) type VolumeName = String;

const REMOVE_OPTIONS: RemoveContainerOptions = RemoveContainerOptions {
    v: true,
    force: true,
    link: false,
};

pub(crate) struct DockerHandler {
    pub(crate) docker: Docker,
    deployment_handler: DeploymentResultHandler,
}

struct VolumeBackup {
    original_name: VolumeName,
    backup_name: VolumeName,
}

impl DockerHandler {
    pub(crate) fn new(deployment_handler: DeploymentResultHandler) -> Self {
        let docker = Docker::connect_with_local_defaults().unwrap();
        Self {
            docker,
            deployment_handler,
        }
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

        let container_config = container_details.clone().config.unwrap();

        let identifier = container_config
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

        let image_identifier = match repo_digests.first() {
            Some(repo_digest) => repo_digest.clone(),
            None => container_config.image.unwrap().clone(),
        };
        debug!("image identifier: {image_identifier}");
        Ok(image_identifier)
    }

    /// Backup volumes by creating copies
    async fn backup_volumes(
        &self,
        container_details: &ContainerInspectResponse,
    ) -> Result<Vec<VolumeBackup>, HoisterError> {
        let mounts = container_details.mounts.as_ref().unwrap_or(&vec![]).clone();

        let mut backups = Vec::new();

        for mount in mounts {
            // Only backup named volumes (not bind mounts)
            if mount.typ != Some(MountPointTypeEnum::VOLUME) {
                continue;
            }

            let volume_name = match &mount.name {
                Some(name) => name.clone(),
                None => continue,
            };

            let backup_name = format!("{}-backup-{}", volume_name, chrono::Utc::now().timestamp());
            info!("Creating volume backup: {} -> {}", volume_name, backup_name);

            // Create backup volume
            let create_options = VolumeCreateOptions {
                name: Some(backup_name.clone()),
                driver: Some(mount.driver.clone().unwrap_or("local".to_string())),
                driver_opts: None,
                labels: None,
                cluster_volume_spec: None,
            };

            self.docker.create_volume(create_options).await?;

            // Copy data from original to backup using a temporary container
            self.copy_volume_data(&volume_name, &backup_name).await?;

            backups.push(VolumeBackup {
                original_name: volume_name,
                backup_name: backup_name.clone(),
            });

            info!("Volume backup created: {}", backup_name);
        }

        Ok(backups)
    }

    /// Remove volume backups
    async fn remove_volume_backups(&self, backups: &[VolumeBackup]) -> Result<(), HoisterError> {
        for backup in backups {
            info!("Removing backup volume: {}", backup.backup_name);
            if let Err(e) = self
                .docker
                .remove_volume(
                    &backup.backup_name,
                    Some(RemoveVolumeOptions { force: true }),
                )
                .await
            {
                warn!(
                    "Failed to remove backup volume {}: {}",
                    backup.backup_name, e
                );
            }
        }
        Ok(())
    }

    /// Restore volumes from backups
    async fn restore_volumes_from_backup(
        &self,
        backups: &[VolumeBackup],
    ) -> Result<(), HoisterError> {
        for backup in backups {
            info!(
                "Restoring volume from backup: {} <- {}",
                backup.original_name, backup.backup_name
            );

            // Remove the failed volume
            if let Err(e) = self
                .docker
                .remove_volume(
                    &backup.original_name,
                    Some(RemoveVolumeOptions { force: true }),
                )
                .await
            {
                warn!("Failed to remove volume {}: {}", backup.original_name, e);
            }

            // Rename backup to original name
            // Note: Docker doesn't have a rename volume command, so we need to:
            // 1. Create new volume with original name
            // 2. Copy data from backup to new volume
            // 3. Remove backup

            let create_options = VolumeCreateOptions {
                name: Some(backup.original_name.clone()),
                driver: Some("local".to_string()),
                driver_opts: None,
                labels: None,
                cluster_volume_spec: None,
            };
            self.docker.create_volume(create_options).await?;

            self.copy_volume_data(&backup.backup_name, &backup.original_name)
                .await?;
            self.docker
                .remove_volume(
                    &backup.backup_name,
                    Some(RemoveVolumeOptions { force: true }),
                )
                .await?;

            info!("Volume restored: {}", backup.original_name);
        }
        Ok(())
    }

    fn is_running_in_container() -> bool {
        // Check for /.dockerenv file (common indicator)
        if Path::new("/.dockerenv").exists() {
            return true;
        }

        // Check if we're in a cgroup that indicates container
        if let Ok(cgroup) = std::fs::read_to_string("/proc/self/cgroup")
            && (cgroup.contains("/docker/") || cgroup.contains("/kubepods/"))
        {
            return true;
        }

        false
    }

    /// Get the current container ID if running in a container
    async fn get_self_container_id(&self) -> Option<String> {
        if !Self::is_running_in_container() {
            return None;
        }

        // Try to read hostname (container ID in Docker)
        if let Ok(hostname) = std::fs::read_to_string("/etc/hostname") {
            let hostname = hostname.trim();

            // Verify this is actually our container by checking with Docker API
            if let Ok(container) = self
                .docker
                .inspect_container(hostname, None::<InspectContainerOptions>)
                .await
            {
                return Some(container.id.unwrap_or(hostname.to_string()));
            }
        }

        None
    }

    /// Copy data between volumes using a temporary container or current container
    async fn copy_volume_data(
        &self,
        source_volume: &str,
        dest_volume: &str,
    ) -> Result<(), HoisterError> {
        debug!("Copying volume data: {} -> {}", source_volume, dest_volume);

        // Check if we're running in a container
        if let Some(self_container_id) = self.get_self_container_id().await {
            debug!(
                "Running in container {}, using self to copy volumes",
                self_container_id
            );
            self.copy_volume_data_using_self(&self_container_id, source_volume, dest_volume)
                .await
        } else {
            debug!("Running on host, using temporary container to copy volumes");
            self.copy_volume_data_using_temp_container(source_volume, dest_volume)
                .await
        }
    }

    /// Copy volumes by mounting them to our own container and executing copy command
    async fn copy_volume_data_using_self(
        &self,
        self_container_id: &str,
        source_volume: &str,
        dest_volume: &str,
    ) -> Result<(), HoisterError> {
        let self_container = self
            .docker
            .inspect_container(self_container_id, None::<InspectContainerOptions>)
            .await?;

        let our_image = self_container
            .config
            .and_then(|c| c.image)
            .unwrap_or_else(|| "alpine:latest".to_string());

        debug!("Using our image for volume copy: {}", our_image);

        let config = ContainerCreateBody {
            image: Some(our_image),
            cmd: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                "cp -a /source/. /dest/".to_string(),
            ]),
            host_config: Some(bollard::models::HostConfig {
                binds: Some(vec![
                    format!("{}:/source:ro", source_volume),
                    format!("{}:/dest", dest_volume),
                ]),
                auto_remove: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        let temp_container = self
            .docker
            .create_container(None::<CreateContainerOptions>, config)
            .await?;

        self.docker
            .start_container(&temp_container.id, None::<StartContainerOptions>)
            .await?;

        // Wait for the container to finish
        let wait_result = self
            .docker
            .wait_container(&temp_container.id, None::<WaitContainerOptions>)
            .try_collect::<Vec<_>>()
            .await;

        match wait_result {
            Ok(results) => {
                if let Some(result) = results.first()
                    && result.status_code != 0
                {
                    let _ = self
                        .docker
                        .remove_container(&temp_container.id, None::<RemoveContainerOptions>)
                        .await;

                    return Err(HoisterError::Docker(format!(
                        "Volume copy failed with status code: {}",
                        result.status_code
                    )));
                }
                debug!("Volume copy completed successfully");
            }
            Err(e) => {
                warn!("Error waiting for temporary container: {}", e);
                // Try to remove the container anyway
                let _ = self
                    .docker
                    .remove_container(&temp_container.id, None::<RemoveContainerOptions>)
                    .await;
            }
        }

        // If auto_remove is true, Docker should clean it up, but let's be explicit
        // This will fail gracefully if already removed
        let _ = self
            .docker
            .remove_container(
                &temp_container.id,
                Some(RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            )
            .await;

        debug!("Temporary container cleaned up");
        Ok(())
    }

    /// Copy volumes using a temporary Alpine container (original method)
    async fn copy_volume_data_using_temp_container(
        &self,
        source_volume: &str,
        dest_volume: &str,
    ) -> Result<(), HoisterError> {
        debug!("Using temporary Alpine container for volume copy");

        // TODO: replace with hoister container to avoid having to pull image
        let config = ContainerCreateBody {
            image: Some("alpine:latest".to_string()),
            cmd: Some(vec![
                "sh".to_string(),
                "-c".to_string(),
                "cp -a /source/. /dest/".to_string(),
            ]),
            host_config: Some(bollard::models::HostConfig {
                binds: Some(vec![
                    format!("{}:/source:ro", source_volume),
                    format!("{}:/dest", dest_volume),
                ]),
                auto_remove: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        };

        let temp_container = self
            .docker
            .create_container(None::<CreateContainerOptions>, config)
            .await?;

        info!("Created temporary container: {}", temp_container.id);
        self.docker
            .start_container(&temp_container.id, None::<StartContainerOptions>)
            .await?;
        info!("Started temporary container: {}", temp_container.id);

        let wait_container_options = WaitContainerOptionsBuilder::new().build();

        //  Err(BollardError(DockerResponseServerError { status_code: 404, message: "No such container: fcec5653c2a00038644b6af614195b458bdaaedf7dc3a699de132efea84eb9f8" }))
        match self
            .docker
            .wait_container(&temp_container.id, Some(wait_container_options))
            .try_collect::<Vec<_>>()
            .await
        {
            Ok(_) => {}
            Err(e) => {
                debug!(
                    "Error waiting for temporary container: {:?}. If this says not found, copy was already done. ignore",
                    e
                );
            }
        }
        debug!("Volume copy completed using temporary container");
        Ok(())
    }

    pub(crate) async fn update_container(
        &self,
        container_id: &ContainerID,
    ) -> Result<(), HoisterError> {
        let image_identifier = self.get_image_identifier(container_id).await?;

        let container_details = self
            .docker
            .inspect_container(container_id, None::<InspectContainerOptions>)
            .await?;

        let old_config = container_details.clone().config.unwrap();
        let image_name = old_config.image.unwrap();

        let (image_name, image_tag) = image_name.rsplit_once(":").unwrap_or_default();

        trace!(
            "container details: {}",
            serde_json::to_string_pretty(&container_details).unwrap()
        );

        debug!("Checking for updates: {image_name}:{image_tag}");

        // Check if volume backup is enabled via label
        let enable_volume_backup = container_details
            .config
            .as_ref()
            .and_then(|c| c.labels.as_ref())
            .and_then(|l| l.get("hoister.backup-volumes"))
            .map(|v| v == "true")
            .unwrap_or(false);

        let new_image_id = download_image(&self.docker, image_name, image_tag).await?;
        debug!("Image pulled successfully (new_image_id: {new_image_id})");

        // Backup volumes if enabled
        let volume_backups = if enable_volume_backup {
            info!("Volume backup enabled, creating backups...");
            self.backup_volumes(&container_details).await?
        } else {
            vec![]
        };

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

        if let Err(_e) = check_container_health(&self.docker, &container.id).await {
            self.deployment_handler
                .inform_container_failed(image_identifier.clone(), container.id.clone())
                .await;
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

            // Restore volumes from backup if they were backed up
            if !volume_backups.is_empty() {
                info!("Restoring volumes from backup...");
                self.restore_volumes_from_backup(&volume_backups).await?;
            }

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
            self.deployment_handler
                .inform_rollback_complete(image_identifier, container_id.clone())
                .await;
        } else {
            debug!("Container updated successfully. deleting old container");
            self.docker
                .remove_container(&backup_name, Some(REMOVE_OPTIONS))
                .await?;

            // Remove volume backups if update was successful
            if !volume_backups.is_empty() {
                info!("Update successful, removing volume backups...");
                self.remove_volume_backups(&volume_backups).await?;
            }

            info!("Container updated successfully. backup container removed");
            self.deployment_handler
                .inform_update_success(image_identifier, container)
                .await;
        }
        Ok(())
    }

    pub(crate) async fn get_containers(
        &self,
        project_name: &str,
    ) -> Result<Vec<ContainerSummary>, Box<dyn Error>> {
        let mut filters = HashMap::new();

        let label_filters = vec![
            "hoister.enable=true".to_string(),
            #[cfg(not(debug_assertions))]
            format!("com.docker.compose.project={}", project_name),
        ];
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
            "found {} containers in project '{}' with label `hoister.enable=true` and not `hoister.hide=true`",
            containers.len(),
            project_name
        );

        Ok(containers)
    }
}

pub(crate) async fn get_project_name(docker: &Docker) -> Result<String, Box<dyn Error>> {
    if let Ok(project_name) = env::var("HOISTER_COMPOSE_PROJECT") {
        info!(
            "Using project name from HOISTER_COMPOSE_PROJECT: {}",
            project_name
        );
        return Ok(project_name);
    }

    let mut filters = HashMap::new();
    filters.insert(
        "label".to_string(),
        vec!["io.hoister.container=agent".to_string()],
    );

    let options = ListContainersOptions {
        filters: Some(filters),
        ..Default::default()
    };

    let containers = docker.list_containers(Some(options)).await?;

    if let Some(container) = containers.first()
        && let Some(labels) = &container.labels
        && let Some(project) = labels.get("com.docker.compose.project")
    {
        info!(
            "Detected project name from hoister agent container: {}",
            project
        );
        return Ok(project.clone());
    }

    let fallback = current_dir()?
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("hoister")
        .to_string();

    Ok(fallback)
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
    let options = CreateImageOptions {
        from_image: Some(image_name.to_owned()),
        tag: Some(image_tag.to_owned()),
        ..Default::default()
    };
    let mut pull_stream = docker.create_image(Some(options), None, Some(CREDENTIALS.clone()));
    while let Some(result) = pull_stream.next().await {
        match result {
            Ok(output) => {
                debug!("{output:?}");
                if let Some(status) = &output.status
                    && (status.contains("Download complete")
                        || status.contains("Pull complete")
                        || status.contains("Downloaded newer image for"))
                {
                    update_available = true;
                }
            }
            Err(e) => error!("Error pulling image: {e:?}"),
        }
    }
    if !update_available {
        return Err(HoisterError::NoUpdateAvailable);
    }

    let full_image_name = format!("{}:{}", image_name, image_tag);
    info!("New image pulled image name image tag: {full_image_name}");
    let image_info = docker
        .inspect_image(&full_image_name)
        .await
        .map_err(|e| HoisterError::Docker(format!("Failed to inspect image: {}", e)))?;
    info!("Image info: {:?}", image_info);
    image_info.id.ok_or(HoisterError::Docker(
        "The pulled image id is empty".to_string(),
    ))
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
