use indoc::indoc;

use opendut_carl_api::carl::CarlClient;
use opendut_model::topology::DeviceId;

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

        let text = match output {
            OutputFormat::Table => {
                format!(indoc!("
                    Device: {}
                      Id: {}
                      Description: {}
                      Interface: {}
                      Tags: [{}]\
                    "),
                    device.name,
                    device.id,
                    device.description.unwrap_or_default(),
                    device.interface,
                    device.tags
                        .iter()
                        .map(|tag| tag.value())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            OutputFormat::Json => {
                serde_json::to_string(&device).unwrap()
            }
            OutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&device).unwrap()
            }
        };
        println!("{text}");

        Ok(())
    }
}
