use opendut_carl_api::carl::CarlClient;

use crate::commands::device::{DeviceTable, render_devices};
use crate::ListOutputFormat;

pub async fn execute(carl: &mut CarlClient, criteria: Vec<String>, output: ListOutputFormat) -> crate::Result<()> {
    let devices = {
        let devices = carl.peers.list_devices().await
            .map_err(|_| String::from("Failed to find devices."))?;

        devices.into_iter()
            .filter(|device| {
                criteria.iter().any(|criterion| {
                    let pattern = glob::Pattern::new(criterion).expect("Failed to read glob pattern");
                    pattern.matches(&device.name.value().to_lowercase())
                        || pattern.matches(&device.id.to_string().to_lowercase())
                        || pattern.matches(&device.description.clone().unwrap().value().to_lowercase())
                        || pattern.matches(&device.interface.to_string().to_lowercase())
                        || device.tags.iter().any(|tag| pattern.matches(&tag.value().to_lowercase()))
                })
            })
            .map(DeviceTable::from)
            .collect::<Vec<_>>()
    };
    let text = render_devices(devices, output);
    println!("{text}");
    Ok(())
}
