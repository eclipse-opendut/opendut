use opendut_carl_api::carl::CarlClient;
use crate::commands::device::{render_devices, SerializableDevice};
use crate::ListOutputFormat;

/// List all devices
#[derive(clap::Parser)]
pub struct ListDevicesCli;

impl ListDevicesCli {
    pub async fn execute(self, carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
        let devices = carl.peers.list_devices().await
            .map_err(|_| String::from("Devices could not be listed"))?
            .into_iter()
            .map(SerializableDevice::from)
            .collect::<Vec<_>>();

        let text = render_devices(devices, output);
        println!("{text}");
        Ok(())
    }
}