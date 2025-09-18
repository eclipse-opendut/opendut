use crate::client::{Client, ContainerId, ContainerInspectInfo, ContainerName, ImageName};
use crate::rt::ContainerRuntimeErrorKind;
use crate::{ContainerRuntimeError, CreateContainerConfig};
use std::sync::{Arc, Mutex, PoisonError};

#[derive(Clone, Debug, Default)]
pub struct MockClient {
    invocations: Arc<Mutex<Vec<MockClientInvocation>>>
}

impl MockClient {

    pub(crate) fn new() -> Self {
        Default::default()
    }

    pub fn invocations(&self) -> Vec<MockClientInvocation> {
        self.invocations.lock()
            .expect("Failed to lock vec of invocations")
            .clone()
    }
}

#[async_trait::async_trait]
impl Client for MockClient {

    async fn create_container(&self, config: CreateContainerConfig) -> Result<ContainerId, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::CreateContainer { config });
        Ok(String::from("<mock-container-id>"))
    }
    
    

    async fn start_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::StartContainer { name });
        Ok(())
    }

    async fn stop_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::StopContainer { name });
        Ok(())
    }

    async fn run_container(&self, config: CreateContainerConfig) -> Result<String, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;

        let create_container_config = Clone::clone(&config);
        let container_image = create_container_config.image;
        let container_name = create_container_config.name.unwrap_or_else(|| String::from("<unknown>"));
        
        invocations.push(MockClientInvocation::PullImage { image: container_image });
        invocations.push(MockClientInvocation::CreateContainer { config });
        invocations.push(MockClientInvocation::StartContainer { name: container_name });
        Ok(String::from("<mock-container-id>"))
    }

    async fn remove_container(&self, name: ContainerName) -> Result<(), ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::RemoveContainer { name });
        Ok(())
    }

    async fn wait_container(&self, name: ContainerName) -> Result<i64, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::WaitContainer { name });
        Ok(0)
    }

    async fn list_containers(&self) -> Result<Vec<String>, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::ListContainers);
        Ok(Vec::new())
    }

    async fn inspect_container(&self, name: ContainerName) -> Result<ContainerInspectInfo, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::InspectContainer { name });
        Ok(ContainerInspectInfo::default())
    }
    
    async fn log_container(&self, name: ContainerName) -> Result<Vec<String>, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::LogContainer { name });
        Ok(Vec::new())
    }
    
    async fn pull_image(&self, image: ImageName) -> Result<String, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::PullImage { image });
        Ok(String::from("<mock-image-id>"))
    }
    async fn remove_image(&self, image: ImageName) -> Result<(), ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::RemoveImage { image });
        Ok(())
    }
    
    async fn list_images(&self) -> Result<Vec<String>, ContainerRuntimeError> {
        let mut invocations = self.invocations.lock()?;
        invocations.push(MockClientInvocation::ListImages);
        Ok(Vec::new())
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
#[cfg_attr(any(test, feature = "mock"), derive(PartialEq))]
pub enum MockClientInvocation {
    CreateContainer { config: CreateContainerConfig },
    StartContainer { name: String },
    StopContainer { name: String },
    RemoveContainer { name: String },
    WaitContainer { name: String },
    ListContainers,
    InspectContainer { name: String },
    LogContainer { name: String },
    PullImage { image: String },
    RemoveImage { image: String },
    ListImages,
}

impl <T> From<PoisonError<T>> for ContainerRuntimeError {
    fn from(value: PoisonError<T>) -> Self {
        ContainerRuntimeError::new(ContainerRuntimeErrorKind::MockLock, value.to_string())
    }
}
