pub mod create;
pub mod describe;
pub mod find;
pub mod delete;
pub mod list;

use cli_table::{Table, WithTitle};
use serde::Serialize;

use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName};

use crate::ListOutputFormat;

#[derive(Table, Serialize)]
struct DeviceTable {
    #[table(title = "Name")]
    name: DeviceName,
    #[table(title = "DeviceID")]
    id: DeviceId,
    #[table(title = "Description")]
    description: DeviceDescription,
    #[table(title = "Tags")]
    tags: String,
}

fn render_devices(devices: Vec<DeviceTable>, output: ListOutputFormat) -> String {
    match output {
        ListOutputFormat::Table => {
            let table = devices
                .with_title()
                .table()
                .display()
                .unwrap();
            format!("{table}")
        }
        ListOutputFormat::Json => {
            serde_json::to_string(&devices).unwrap()
        }
        ListOutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&devices).unwrap()
        }
    }
}

impl From<DeviceDescriptor> for DeviceTable {
    fn from(device: DeviceDescriptor) -> Self {
        DeviceTable {
            name: device.name,
            id: device.id,
            description: device.description.unwrap_or_default(),
            tags: device.tags.iter().map(|tag| tag.value()).collect::<Vec<_>>().join(", "),
        }
    }
}
