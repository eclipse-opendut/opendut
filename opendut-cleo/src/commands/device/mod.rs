pub mod create;
pub mod describe;
pub mod find;
pub mod delete;
pub mod list;

use cli_table::{print_stdout, Table, WithTitle};
use serde::Serialize;
use opendut_carl_api::carl::CarlClient;
use opendut_model::topology::{DeviceDescriptor, DeviceId, DeviceName};
use opendut_model::util::net::NetworkInterfaceName;
use crate::OutputFormat;

async fn render_devices(devices: Vec<DeviceDescriptor>, carl: &mut CarlClient, output: OutputFormat) -> crate::Result<()> {

    let peers = carl.peers.list_peer_descriptors().await
        .map_err(|error| format!("Could not list peers.\n  {error}"))?;

    let interface_names_by_device = peers.iter()
        .flat_map(|peer| {
            peer.topology.devices.iter().filter_map(|device| {
                let interface = peer.network.interfaces.iter()
                    .find(|interface| interface.id == device.interface)?;

                Some((device.id, interface.name.clone()))
            })
        })
        .collect::<std::collections::HashMap<_, _>>();

    match output {
        OutputFormat::Table => {
            let device_table = devices.into_iter()
                .map(|device| {
                    let network_interface = interface_names_by_device
                        .get(&device.id)
                        .cloned()
                        .ok_or_else(|| format!(
                            "Could not find network interface name for device <{}>.",
                            device.id
                        ))?;

                    let DeviceDescriptor { name, id, tags, description: _, interface: _ } = device;

                    Ok(DeviceTable {
                        name,
                        id,
                        network_interface,
                        tags: tags.iter().map(|tag| tag.value()).collect::<Vec<_>>().join(", "),
                    })
                }).collect::<Result<Vec<_>, String>>()?;

            print_stdout(device_table.with_title())
                .expect("List of devices should be printable as table.");
        }
        OutputFormat::Json => {
            let json = serde_json::to_string(&devices)
                .map_err(|error| format!("Could not serialize devices.\n  {error}"))?;
            println!("{json}");
        }
        OutputFormat::PrettyJson => {
            let json = serde_json::to_string_pretty(&devices)
                .map_err(|error| format!("Could not serialize devices.\n  {error}"))?;
            println!("{json}");
        }
    }
    Ok(())
}

#[derive(Table, Serialize)]
struct DeviceTable {
    #[table(title = "Name")]
    name: DeviceName,
    #[table(title = "DeviceID")]
    id: DeviceId,
    #[table(title = "NetworkInterface")]
    network_interface: NetworkInterfaceName,
    #[table(title = "Tags")]
    tags: String,
}
