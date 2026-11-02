use indoc::formatdoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::topology::{DeviceDescriptor, DeviceId};

use crate::OutputFormat;

/// Describe a device
#[derive(clap::Parser)]
pub struct DescribeDeviceCli {
    /// ID of the device
    #[arg()]
    id: DeviceId,
}

impl DescribeDeviceCli {
    pub async fn execute(self, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {
        let device_id = self.id;

        let devices = carl.peers.list_devices().await
            .map_err(|_| String::from("Failed to fetch list of devices."))?;

        let device = devices.into_iter().find(|device| device.id == device_id)
            .ok_or(format!("Failed to find device for id <{device_id}>"))?;

        render_device_descriptor(device, output)
    }
}

fn render_device_descriptor(device: DeviceDescriptor, output: OutputFormat) -> crate::Result<()> {
    let text = match output {
        OutputFormat::Table => {
            let device_id = device.id;
            let name = device.name;
            let description = device.description.unwrap_or_default();
            let interface = device.interface;
            let tags = device.tags.iter()
                .map(|tag| tag.value())
                .collect::<Vec<_>>()
                .join(", ");

            formatdoc!("
                Device: {name}
                  Id: {device_id}
                  Description: {description}
                  Interface: {interface}
                  Tags: [{tags}]\
            ")
        }
        OutputFormat::Json => {
            serde_json::to_string(&device)
                .map_err(|error| format!("Could not serialize device.\n  {error}"))?
        }
        OutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&device)
                .map_err(|error| format!("Could not serialize device.\n  {error}"))?
        }
    };

    println!("{text}");
    Ok(())
}
