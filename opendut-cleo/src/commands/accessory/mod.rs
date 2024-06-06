pub mod create;
pub mod delete;
pub mod list;

use cli_table::{Table, WithTitle};
use serde::Serialize;

use opendut_types::topology::{AccessoryDescription, AccessoryDescriptor, AccessoryId, AccessoryName, AccessoryModel};

use crate::ListOutputFormat;

#[derive(Table, Serialize)]
struct AccessoryTable {
    #[table(title = "Name")]
    name: AccessoryName,
    #[table(title = "AccessoryID")]
    id: AccessoryId,
    #[table(title = "Description")]
    description: AccessoryDescription,
    #[table(title = "Model")]
    model: AccessoryModel,
}

fn render_accessories(accessories: Vec<AccessoryTable>, output: ListOutputFormat) -> String {
    match output {
        ListOutputFormat::Table => {
            let table = accessories
                .with_title()
                .table()
                .display()
                .unwrap();
            format!("{table}")
        }
        ListOutputFormat::Json => {
            serde_json::to_string(&accessories).unwrap()
        }
        ListOutputFormat::PrettyJson => {
            serde_json::to_string_pretty(&accessories).unwrap()
        }
    }
}

impl From<AccessoryDescriptor> for AccessoryTable {
    fn from(device: AccessoryDescriptor) -> Self {
        AccessoryTable {
            name: device.name,
            id: device.id,
            description: device.description.unwrap_or_default(),
            model: device.model,
        }
    }
}
