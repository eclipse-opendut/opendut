use std::sync::Arc;

use opendut_types::topology::{AccessoryDescriptor, AccessoryModel};
use tokio::sync::Mutex;
use crate::service::accessory::Accessory;
use crate::service::accessory::manson_hcs3304::MansonHcs3304;

use tokio::{sync::watch, task::JoinHandle};
use tokio::time::{Duration, Instant, sleep};
use tracing::warn;


const MAX_ACCESSORY_TERMINATION_TIME: Duration = std::time::Duration::from_millis(1000);

type AccessoryHandle = (AccessoryDescriptor, JoinHandle<()>, watch::Sender<bool>);

pub type AccessoryManagerRef = Arc<AccessoryManager>;

pub struct AccessoryManager {
    accessories: Mutex<Vec<AccessoryHandle>>
}

impl AccessoryManager {
    pub fn create() -> AccessoryManagerRef {
        Arc::new(
            Self { accessories: Mutex::new(Vec::new()) }
        )
    }

    pub async fn deploy_accessory(&self, descriptor: AccessoryDescriptor, mqtt_broker_url: Option<url::Url>) {
        let (tx_termination_channel, rx_termination_channel) = watch::channel(false);

        let mut accessory = match &descriptor.model {
            AccessoryModel::MansonHcs3304 { serial_port } => {
                MansonHcs3304::new(rx_termination_channel, serial_port.clone(), mqtt_broker_url)
            },
        };

        let join_handle: JoinHandle<()> = tokio::spawn(async move {
            accessory.deploy()
        });

        self.accessories.lock().await.push(
            (
                descriptor,
                join_handle,
                tx_termination_channel
            )
        );
    }

    pub async fn undeploy_accessories(&self) {
        let mut accessories = self.accessories.lock().await;
        
        for (descriptor, _join_handle, tx_termination_channel ) in accessories.iter() {
            if let Err(cause) = tx_termination_channel.send(true) {
                warn!("Failed to send termination signal to accessory '{}', perhaps it already terminated? Cause: {}", descriptor.name, cause);
            }
        }
        let t_start = Instant::now();

        while let Some((descriptor, join_handle, _tx_termination_channel )) = accessories.pop() {
            tokio::select! {
                _ = join_handle => {}
                _ = sleep(MAX_ACCESSORY_TERMINATION_TIME - t_start.elapsed()) => {
                    warn!("Task for accessory '{}' has not terminated within {} seconds after requesting termination.", descriptor.name, MAX_ACCESSORY_TERMINATION_TIME.as_secs())
                }
            };
        }

    }
}