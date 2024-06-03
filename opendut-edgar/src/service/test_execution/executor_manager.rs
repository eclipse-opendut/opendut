use std::sync::{Arc, Mutex};

use opendut_types::peer::executor::ExecutorDescriptor;
use tokio::sync::watch::{self, Sender};
use tracing::warn;

use crate::service::test_execution::container_manager::{ContainerManager, ContainerConfiguration};

pub type ExecutorManagerRef = Arc<Mutex<ExecutorManager>>;

pub struct ExecutorManager {
    tx_termination_channels: Vec<Sender<bool>>,
}

impl ExecutorManager {
    pub fn create() -> ExecutorManagerRef {
        Arc::new(Mutex::new(Self {
            tx_termination_channels: Vec::new(),
        }))
    }

    pub fn create_new_executor(&mut self, executor_descriptor: ExecutorDescriptor) {
        let (tx, rx) = watch::channel(false);

        match executor_descriptor {
            ExecutorDescriptor::Executable => warn!("Executing Executable not yet implemented."),
            ExecutorDescriptor::Container {
                engine,
                name,
                image,
                volumes: _,
                devices: _,
                envs,
                ports: _,
                command,
                args,
                results_url
            } => {
                let container_config = ContainerConfiguration{
                    name,
                    engine,
                    image,
                    command,
                    args,
                    envs,
                    results_url,
                };
                tokio::spawn(async move {
                    ContainerManager::new(container_config, rx).start().await;
                });
            }
        }
        self.tx_termination_channels.push(tx);
    }

    pub fn terminate_executors(&mut self) {
        for tx_termination_channel in &self.tx_termination_channels {
            if let Err(cause) = tx_termination_channel.send(true) {
                warn!("Failed to send termination signal to executor, perhaps it already terminated? Cause: {cause}");
            }
        }
        self.tx_termination_channels.clear();

    }


}
