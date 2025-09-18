use std::rc::Rc;
use crate::client::{Client, ContainerId, ContainerInspectInfo, ContainerName, CreateContainerConfig, ImageName};

#[derive(Clone)]
pub struct ContainerRuntime {
    client: Rc<dyn Client>,
}

impl ContainerRuntime
{
    #[cfg(feature = "docker")]
    pub fn new_docker() -> Result<ContainerRuntime, ContainerRuntimeError> {
        let client = crate::client::docker::DockerClient::new()?;
        Ok(ContainerRuntime {
            client: Rc::new(client),
        })
    }

    #[cfg(any(test, feature = "mock"))]
    pub fn new_mock() -> (crate::client::mock::MockClient, ContainerRuntime) {
        let mock = crate::client::mock::MockClient::new();
        (Clone::clone(&mock), ContainerRuntime { client: Rc::new(mock) })
    }

    pub async fn create_container(&self, config: CreateContainerConfig) -> Result<ContainerId, ContainerRuntimeError> {
        self.client.create_container(config).await
    }

    pub async fn start_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        self.client.start_container(name).await
    }

    pub async fn stop_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        self.client.stop_container(name).await
    }
    
    pub async fn run_container(&self, config: CreateContainerConfig) -> Result<String, ContainerRuntimeError> {
        self.client.run_container(config).await
    }

    pub async fn remove_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        self.client.remove_container(name).await
    }

    pub async fn wait_container(&self, name: ContainerName) -> Result<i64, ContainerRuntimeError> {
        self.client.wait_container(name).await
    }
    
    pub async fn list_containers(&self) -> Result<Vec<String>, ContainerRuntimeError> {
        self.client.list_containers().await
    }
    
    pub async fn inspect_container(&self, name: ContainerName) -> Result<ContainerInspectInfo, ContainerRuntimeError> {
        self.client.inspect_container(name).await
    }
    
    pub async fn log_container(&self, name: ContainerName) -> Result<Vec<String>, ContainerRuntimeError> {
        self.client.log_container(name).await
    }
    
    pub async fn pull_image(&self, image_name: ImageName) -> Result<String, ContainerRuntimeError> {
        self.client.pull_image(image_name).await
    }

    pub async fn remove_image(&self, image: ImageName) -> Result<(), ContainerRuntimeError> {
        self.client.remove_image(image).await
    }
    
    pub async fn list_images(&self) -> Result<Vec<String>, ContainerRuntimeError> {
        self.client.list_images().await
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct ContainerRuntimeError {
    pub kind: ContainerRuntimeErrorKind,
    pub message: String,
    pub affected_container: Option<ContainerName>,
}

impl ContainerRuntimeError {

    pub fn new(kind: ContainerRuntimeErrorKind, message: String) -> ContainerRuntimeError {
        ContainerRuntimeError {
            kind,
            message,
            affected_container: None,
        }
    }

    pub fn new_with_container(kind: ContainerRuntimeErrorKind, message: String, container: ContainerName) -> ContainerRuntimeError {
        ContainerRuntimeError {
            kind,
            message,
            affected_container: Some(container),
        }
    }
}

#[derive(Clone, Debug)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub enum ContainerRuntimeErrorKind {
    Initialization,
    CreateContainer,
    StartContainer,
    StopContainer,
    RunContainer,
    RemoveContainer,
    WaitContainer,
    ListContainers,
    InspectContainer,
    LogContainer,
    PullImage,
    RemoveImage,
    ListImages,
    
    #[cfg(feature = "mock")] MockLock,
}

impl core::error::Error for ContainerRuntimeError {}

impl std::fmt::Display for ContainerRuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "An Error occurred ")?;
        let affected_container = self.affected_container.as_ref()
            .map(|id| id.to_string())
            .unwrap_or_else(|| String::from("<unknown>"));
        match self.kind {
            ContainerRuntimeErrorKind::Initialization => write!(f, "during runtime initialization")?,
            ContainerRuntimeErrorKind::CreateContainer => write!(f, "when creating container '{affected_container}'")?,
            ContainerRuntimeErrorKind::StartContainer => write!(f, "when starting container '{affected_container}'")?,
            ContainerRuntimeErrorKind::StopContainer => write!(f, "when stopping container '{affected_container}'")?,
            ContainerRuntimeErrorKind::RunContainer => write!(f, "when running container for container {affected_container}")?,
            ContainerRuntimeErrorKind::RemoveContainer => write!(f, "when removing container '{affected_container}'")?,
            ContainerRuntimeErrorKind::WaitContainer => write!(f, "when waiting for container '{affected_container}' to terminate")?,
            ContainerRuntimeErrorKind::ListContainers => write!(f, "when listing containers ({affected_container})")?,
            ContainerRuntimeErrorKind::InspectContainer => write!(f, "when inspecting container '{affected_container}'")?,
            ContainerRuntimeErrorKind::LogContainer => write!(f, "when logging container '{affected_container}'")?,
            ContainerRuntimeErrorKind::PullImage => write!(f, "when pulling image for container {affected_container}")?,
            ContainerRuntimeErrorKind::RemoveImage => write!(f, "when removing image for container {affected_container}")?,
            ContainerRuntimeErrorKind::ListImages => write!(f, "when listing images for container {affected_container}")?,
            #[cfg(feature = "mock")] ContainerRuntimeErrorKind::MockLock => write!(f, "when accessing the inner lock of the mock")?
        };
        write!(f, ": {}", self.message)
    }
}
