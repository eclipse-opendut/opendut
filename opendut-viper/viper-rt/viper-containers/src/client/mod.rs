use crate::ContainerRuntimeError;

#[cfg(feature = "docker")]
pub mod docker;

#[cfg(feature = "mock")]
pub mod mock;

#[async_trait::async_trait]
pub trait Client {

    async fn create_container(&self, config: CreateContainerConfig) -> Result<ContainerId, ContainerRuntimeError>;
    
    async fn start_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError>;

    async fn stop_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError>;
    
    async fn run_container(&self, config: CreateContainerConfig) -> Result<String, ContainerRuntimeError>;

    async fn remove_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError>;

    async fn wait_container(&self, name: ContainerName) -> Result<i64, ContainerRuntimeError>;

    async fn list_containers(&self) -> Result<Vec<String>, ContainerRuntimeError>;

    async fn inspect_container(&self, name: ContainerName) -> Result<ContainerInspectInfo, ContainerRuntimeError>;
    
    async fn log_container(&self, name: ContainerName) -> Result<Vec<String>, ContainerRuntimeError>;
    
    async fn pull_image(&self, image: ImageName) -> Result<String, ContainerRuntimeError>;

    async fn remove_image(&self, image: ImageName) -> Result<(), ContainerRuntimeError>;
    
    async fn list_images(&self) -> Result<Vec<String>, ContainerRuntimeError>;

}

pub type ContainerId = String;
pub type ContainerName = String;
pub type ContainerImage = String;
pub type ImageName = String;

#[derive(Clone, Debug)]
#[cfg_attr(any(test, feature = "mock"), derive(PartialEq))]
pub struct CreateContainerConfig {
    pub name: Option<ContainerName>,
    pub image: ContainerImage,
    pub command: Option<Vec<String>>,
    pub entrypoint: Option<Vec<String>>,
    pub env: Option<Vec<String>>,
    pub tty: Option<bool>,
    pub open_stdin: Option<bool>,
    pub user: Option<String>,
    pub volumes: Vec<VolumeMountConfig>,
    pub network: Option<String>
}

#[derive(Clone, Debug)]
#[cfg_attr(any(test, feature = "mock"), derive(PartialEq))]
pub struct VolumeMountConfig {
    pub src: String,
    pub dst: String,
}

#[derive(Clone, Debug)]
#[cfg_attr(any(test, feature = "mock"), derive(PartialEq))]
pub struct StartContainerConfig {}

#[derive(Debug)]
pub enum ContainerStateStatus {
    Empty,
    Created,
    Running,
    Paused,
    Restarting,
    Removing,
    Exited,
    Dead,
}

#[derive(Debug)]
pub struct InspectedContainerState {
    pub status: ContainerStateStatus,
    pub running: bool,
    pub paused: bool,
    pub restarting: bool,
    pub oom_killed: bool,
    pub dead: bool,
    pub exit_code: i64,
}

impl Default for InspectedContainerState {
    fn default() -> Self {
        Self {
            status: ContainerStateStatus::Empty,
            running: false,
            paused: false,
            restarting: false,
            oom_killed: false,
            dead: false,
            exit_code: 0,
        }
    }
}

#[derive(Debug, Default)]
pub struct ContainerInspectInfo {
    pub id: String,
    pub name: String,
    pub created: String,
    pub path: String,
    pub args: Vec<String>,
    pub state: InspectedContainerState,
    pub restart_count: i64,
}
