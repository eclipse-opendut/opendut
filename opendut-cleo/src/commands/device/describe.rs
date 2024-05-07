use indoc::indoc;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::topology::{DeviceDescription, DeviceId};

use crate::DescribeOutputFormat;

/// Describe a device
#[derive(clap::Parser)]
pub struct DescribeDeviceCli {
    ///DeviceID
    #[arg()]
    id: Uuid,
}

impl DescribeDeviceCli {
    pub async fn execute(self, carl: &mut CarlClient, output: DescribeOutputFormat) -> crate::Result<()> {
        let device_id = DeviceId::from(self.id);

        let devices = carl.peers.list_devices().await
            .map_err(|_| String::from("Failed to fetch list of devices."))?;

        let device = devices.into_iter().find(|device| device.id == device_id)
            .ok_or(format!("Failed to find device for id <{}>", device_id))?;

        let text = match output {
            DescribeOutputFormat::Text => {
                format!(indoc!("
                Device: {}
                  Id: {}
                  Description: {}
                  Interface: {}
                  Tags: [{}]\
            "),
                        device.name,
                        device.id,
                        device.description
                            .map(DeviceDescription::from)
                            .unwrap_or_default(),
                        device.interface,
                        device.tags
                            .iter()
                            .map(|tag| tag.value())
                            .collect::<Vec<_>>()
                            .join(", "))
            }
            DescribeOutputFormat::Json => {
                serde_json::to_string(&device).unwrap()
            }
            DescribeOutputFormat::PrettyJson => {
                serde_json::to_string_pretty(&device).unwrap()
            }
        };
        println!("{text}");

        Ok(())
    }
}
