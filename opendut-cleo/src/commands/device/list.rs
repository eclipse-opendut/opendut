use opendut_carl_api::carl::CarlClient;
use crate::commands::device::render_devices;
use crate::OutputFormat;

/// List all devices
#[derive(clap::Parser)]
pub struct ListDevicesCli;

impl ListDevicesCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let devices = carl.peers.list_devices().await
            .map_err(|_| String::from("Devices could not be listed"))?;
        
        render_devices(devices, carl, output).await?;
        Ok(())
    }
}