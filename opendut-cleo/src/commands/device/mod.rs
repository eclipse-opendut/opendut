pub mod create;
pub mod describe;
pub mod find;
pub mod delete;
pub mod list;

use cli_table::{Table, WithTitle};
use serde::Serialize;

use opendut_model::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName};

use crate::ListOutputFormat;

fn render_devices(devices: Vec<SerializableDevice>, output: ListOutputFormat) -> String {
    match output {
        ListOutputFormat::Table => {
            let devices = devices.into_iter()
                .map(DeviceTable::from)
                .collect::<Vec<_>>();

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

#[derive(Serialize)]
struct SerializableDevice {
    name: DeviceName,
    id: DeviceId,
    description: DeviceDescription,
    tags: Vec<String>,
}

impl From<DeviceDescriptor> for SerializableDevice {
    fn from(device: DeviceDescriptor) -> Self {
        SerializableDevice {
            name: device.name,
            id: device.id,
            description: device.description.unwrap_or_default(),
            tags: device.tags.into_iter()
                .map(|tag| tag.value().to_owned())
                .collect::<Vec<_>>(),
        }
    }
}

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

impl From<SerializableDevice> for DeviceTable {
    fn from(device: SerializableDevice) -> Self {
        let SerializableDevice { name, id, description, tags } = device;

        DeviceTable {
            name,
            id,
            description,
            tags: tags.join(", "),
        }
    }
}
