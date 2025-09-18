use bollard::container::LogOutput;
use crate::client::*;
use crate::rt::ContainerRuntimeErrorKind;
use crate::ContainerRuntimeError;
use bollard::models::*;
use bollard::query_parameters::*;
use futures::StreamExt;

pub struct DockerClient {
    docker: bollard::Docker,
}

impl DockerClient {
    pub fn new() -> Result<Self, ContainerRuntimeError> {
        let docker = bollard::Docker::connect_with_defaults()
            .map_err(|err| ContainerRuntimeError::new(ContainerRuntimeErrorKind::Initialization, err.to_string()))?;
        Ok(Self {
            docker,
        })
    }
}

#[async_trait::async_trait]
impl Client for DockerClient {

    async fn create_container(&self, config: CreateContainerConfig) -> Result<ContainerId, ContainerRuntimeError> {
        let container_name = Clone::clone(&config.name);
        let options = CreateContainerOptions {
            name: Clone::clone(&container_name),
            ..Default::default()
        };
        let response = self.docker.create_container(Some(options), config).await
            .map_err(|err| ContainerRuntimeError::new_with_container(ContainerRuntimeErrorKind::CreateContainer, err.to_string(), container_name.unwrap_or_else(|| String::from("<unknown>"))))?;

        Ok(ContainerId::from(response.id))
    }

    async fn start_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        let _ = self.docker.start_container(&name, None::<StartContainerOptions>).await
            .map_err(|err| ContainerRuntimeError::new_with_container(ContainerRuntimeErrorKind::StartContainer, err.to_string(), name))?;
        Ok(())
    }

    async fn stop_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        #[allow(deprecated)] // TODO: Remove if bollard provides an alternative.
        let _ = self.docker.stop_container(&name, None::<bollard::container::StopContainerOptions>).await
            .map_err(|err| ContainerRuntimeError::new_with_container(ContainerRuntimeErrorKind::StopContainer, err.to_string(), name))?;
        Ok(())
    }

    async fn run_container(&self, config: CreateContainerConfig) -> Result<String, ContainerRuntimeError> {
        let create_container_config = Clone::clone(&config);
        let container_image = create_container_config.image;
        let container_name = create_container_config.name.unwrap_or_else(|| String::from("<unknown>"));
        
        self.pull_image(container_image).await?;
        let container_id = self.create_container(config).await?;
        self.start_container(container_name).await?;
        
        Ok(ContainerId::from(container_id))
    }

    async fn remove_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        let _ = self.docker.remove_container(&name, None::<RemoveContainerOptions>).await
            .map_err(|err| ContainerRuntimeError::new_with_container(ContainerRuntimeErrorKind::RemoveContainer, err.to_string(), name))?;
        Ok(())
    }

    async fn wait_container(&self, name: ContainerName) -> Result<i64, ContainerRuntimeError> {
        let mut stream = self.docker.wait_container(&name, Some(WaitContainerOptions::default()));
        let Some(result) = stream.next().await else { // TODO: There should be a timeout.
            return Err(ContainerRuntimeError::new(ContainerRuntimeErrorKind::WaitContainer, String::from("Did not get a reply.")))
        };
        result
            .map(|response| response.status_code)
            .map_err(|error| ContainerRuntimeError::new(ContainerRuntimeErrorKind::WaitContainer, error.to_string()))
    }

    async fn list_containers(&self) -> Result<Vec<String>, ContainerRuntimeError> {
        let options = ListContainersOptions {
            all: true,
            ..Default::default()
        };

        let containers = self.docker.list_containers(Some(options)).await;

        match containers {
            Ok(containers) => {
                let mut result: Vec<String> = Vec::new();
                for container in containers {
                    result.push(container.id.unwrap_or_else(|| String::from("<unknown>")));
                }
                Ok(result)
            }
            Err(err) => Err(ContainerRuntimeError::new(ContainerRuntimeErrorKind::ListContainers, err.to_string()))
        }
    }

    async fn inspect_container(&self, name: ContainerName) -> Result<ContainerInspectInfo, ContainerRuntimeError> {

        let container_info = self.docker.inspect_container(name.as_str(), Some(InspectContainerOptions::default())).await;

        match container_info {
            Ok(info) => {

                let bollard_state = info.state.unwrap_or_default();

                let missing_field_error = |field: &str| ContainerRuntimeError::new_with_container(
                    ContainerRuntimeErrorKind::InspectContainer,
                    format!("Missing field '{field}'"),
                    Clone::clone(&name)
                );

                let state = InspectedContainerState {
                    status: get_container_state_status(bollard_state.status),
                    running: bollard_state.running.ok_or_else(|| missing_field_error("state.running"))?,
                    paused: bollard_state.paused.ok_or_else(|| missing_field_error("state.paused"))?,
                    restarting: bollard_state.restarting.ok_or_else(|| missing_field_error("state.restarting"))?,
                    oom_killed: bollard_state.oom_killed.ok_or_else(|| missing_field_error("state.oom_killed"))?,
                    dead: bollard_state.dead.ok_or_else(|| missing_field_error("state.dead"))?,
                    exit_code: bollard_state.exit_code.ok_or_else(|| missing_field_error("state.exit_code"))?,
                };

                Ok(ContainerInspectInfo {
                    id: info.id.ok_or_else(|| missing_field_error("id"))?,
                    name: info.name.ok_or_else(|| missing_field_error("name"))?,
                    created: info.created.ok_or_else(|| missing_field_error("created"))?,
                    path: info.path.ok_or_else(|| missing_field_error("path"))?,
                    args: info.args.ok_or_else(|| missing_field_error("args"))?,
                    state,
                    restart_count: info.restart_count.ok_or_else(|| missing_field_error("restart_count"))?,
                })
            }
            Err(err) => {
                Err(ContainerRuntimeError::new_with_container(ContainerRuntimeErrorKind::InspectContainer, err.to_string(), name))
            }
        }
    }

    async fn log_container(&self, name: ContainerName) -> Result<Vec<String>, ContainerRuntimeError> {

        let options = LogsOptions {
            stdout: true,
            stderr: true,
            ..Default::default()
        };

        let mut logs_stream = self.docker.logs(name.as_str(), Some(options));

        let mut logs = Vec::new();

        while let Some(log) = logs_stream.next().await {
            if let Ok(log) = log {
                match log {
                    LogOutput::StdOut { message } |
                    LogOutput::StdErr { message } |
                    LogOutput::Console { message } => {
                        logs.push(String::from_utf8_lossy(&message).to_string());
                    }
                    LogOutput::StdIn { .. } => {}
                }
            }
        }

        Ok(logs)
    }

    async fn pull_image(&self, image: ImageName) -> Result<String, ContainerRuntimeError> {
        
        let options = CreateImageOptions {
            from_image: Some(Clone::clone(&image)),
            ..Default::default()
        };

        let mut image_id = String::new();

        let mut stream = self.docker.create_image(Some(options), None, None);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    image_id = response.id.unwrap_or_else(|| String::from("<unknown image id>"));
                }
                Err(err) => {
                    return Err(ContainerRuntimeError::new(ContainerRuntimeErrorKind::PullImage, err.to_string()));
                }
            }
        }
        Ok(image_id)
    }

    async fn remove_image(&self, image: ImageName) -> Result<(), ContainerRuntimeError> {
        let options = RemoveImageOptions {
            force: false,
            noprune: false,
        };

        let result = self.docker.remove_image(image.as_str(), Some(options), None).await;

        match result {
            Ok(_) => {
                Ok(())
            }
            Err(err) => {
                Err(ContainerRuntimeError::new(ContainerRuntimeErrorKind::RemoveImage, err.to_string()))
            }
        }
    }

    async fn list_images(&self) -> Result<Vec<String>, ContainerRuntimeError> {
        
        let options = ListImagesOptions {
            all: true,
            ..Default::default()
        };
        let images = self.docker.list_images(Some(options)).await;
        
        match images {
            Ok(images) => {
                let mut image_ids = Vec::new();
                for image in images {
                    image_ids.push(image.id);
                }
                Ok(image_ids)
            }
            Err(err) => Err(ContainerRuntimeError::new(ContainerRuntimeErrorKind::ListImages, err.to_string()))
        }
    }
}

impl From<CreateContainerConfig> for ContainerCreateBody {
    fn from(config: CreateContainerConfig) -> Self {
        
        let mounts = config.volumes.into_iter().map(|volume| {
            Mount {
                target: Some(volume.dst),
                source: Some(volume.src),
                typ: Some(MountTypeEnum::BIND),
                consistency: Some(String::from("default")),
                ..Default::default()
            }
        }).collect::<Vec<_>>();
        
        let host_config = HostConfig {
            mounts: Some(mounts),
            network_mode: config.network,
            ..Default::default()
        };
        
        ContainerCreateBody {
            image: Some(config.image),
            cmd: config.command,
            entrypoint: config.entrypoint,
            env: config.env,
            tty: config.tty,
            open_stdin: config.open_stdin,
            user: config.user,
            host_config: Some(host_config),
            ..Default::default()
        }
    }
}

fn get_container_state_status(variant: Option<ContainerStateStatusEnum>) -> ContainerStateStatus {

    match variant.expect("No ContainerStateStatus found. ") {
        ContainerStateStatusEnum::EMPTY => ContainerStateStatus::Empty,
        ContainerStateStatusEnum::CREATED => ContainerStateStatus::Created,
        ContainerStateStatusEnum::RUNNING => ContainerStateStatus::Running,
        ContainerStateStatusEnum::PAUSED => ContainerStateStatus::Paused,
        ContainerStateStatusEnum::RESTARTING => ContainerStateStatus::Restarting,
        ContainerStateStatusEnum::REMOVING => ContainerStateStatus::Removing,
        ContainerStateStatusEnum::EXITED => ContainerStateStatus::Exited,
        ContainerStateStatusEnum::DEAD => ContainerStateStatus::Dead,
    }
}
