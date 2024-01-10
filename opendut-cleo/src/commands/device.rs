use cli_table::{Table, WithTitle};
use serde::Serialize;

use opendut_carl_api::carl::CarlClient;
use opendut_types::topology::{Device, DeviceId};

use crate::ListOutputFormat;

#[derive(Table, Serialize)]
struct DeviceTable {
    #[table(title = "Name")]
    name: String,
    #[table(title = "DeviceID")]
    id: DeviceId,
    #[table(title = "Location")]
    location: String,
    #[table(title = "Description")]
    description: String,
    #[table(title = "Tags")]
    tags: String,
}

pub async fn list_devices(carl: &mut CarlClient, output: ListOutputFormat) -> crate::Result<()> {
    let devices = carl.peers.list_devices().await
        .map_err(|_| String::from("Devices could not be listed"))?
        .into_iter()
        .map(DeviceTable::from)
        .collect::<Vec<_>>();

    let text = render_devices(devices, output);
    println!("{text}");
    Ok(())
}

pub mod create {
    use uuid::Uuid;
    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerId};
    use opendut_types::topology::{Device, DeviceId};
    use opendut_types::util::net::NetworkInterfaceName;
    use crate::DescribeOutputFormat;

    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        carl: &mut CarlClient,
        peer_id: Uuid,
        device_id: Option<Uuid>,
        name: Option<String>,
        description: Option<String>,
        location: Option<String>,
        interface: Option<NetworkInterfaceName>,
        tags: Option<Vec<String>>
    ) -> crate::Result<()> {
        let peer_id = PeerId::from(peer_id);
        let device_id = device_id.map(DeviceId::from).unwrap_or(DeviceId::random());

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;
        let maybe_existing_device = peer_descriptor.topology.devices.iter_mut().find(|device| device.id == device_id) ;
        match maybe_existing_device {
            None => {
                //TODO provide separate `update device` command to not need this custom input handling
                let name = name.ok_or(String::from("Cannot create new device because of missing device name."))?;
                let interface = interface.ok_or(String::from("Cannot create new device because of missing interface name."))?;

                let new_device = Device {
                    id: device_id,
                    name,
                    description: description.unwrap_or_default(),
                    location: location.unwrap_or_default(),
                    interface,
                    tags: tags.unwrap_or_default(),
                };
                peer_descriptor.topology.devices.push(new_device);
            }
            Some(device) => {
                if let Some(name) = name {
                    device.name = name;
                }
                if let Some(description) = description {
                    device.description = description; 
                }
                if let Some(location) = location {
                    device.location = location;
                }
                if let Some(interface) = interface {
                    device.interface = interface;
                }
                if let Some(tags) = tags {
                    device.tags = tags;
                }
            }
        }
        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, DescribeOutputFormat::Text);
        
        Ok(())
    }
}

pub mod describe {
    use indoc::indoc;
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::topology::DeviceId;

    use crate::DescribeOutputFormat;

    pub async fn execute(carl: &mut CarlClient, id: Uuid, output: DescribeOutputFormat) -> crate::Result<()> {
        let device_id = DeviceId::from(id);

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
                      Location: {}
                      Interface: {}
                      Tags: [{}]\
                "), device.name, device.id, device.description, device.location, device.interface, device.tags.join(", "))
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

pub mod find {
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
                        pattern.matches(&device.name.to_lowercase())
                            || pattern.matches(&device.id.to_string().to_lowercase())
                            || pattern.matches(&device.description.to_lowercase())
                            || pattern.matches(&device.interface.to_string().to_lowercase())
                            || pattern.matches(&device.location.to_lowercase())
                            || device.tags.iter().any(|tag| pattern.matches(&tag.to_lowercase()))
                    })
                })
                .map(DeviceTable::from)
                .collect::<Vec<_>>()
        };
        let text = render_devices(devices, output);
        println!("{text}");
        Ok(())
    }
}

pub mod delete {
    use uuid::Uuid;
    use opendut_carl_api::carl::CarlClient;
    use opendut_types::topology::DeviceId;

    pub async fn execute(carl: &mut CarlClient, device_id: Uuid) -> crate::Result<()> {
        let device_id = DeviceId::from(device_id);

        let mut peers = carl.peers.list_peer_descriptors().await
            .map_err(|error| format!("Could not list peers.\n  {}", error))?;

        let peer = peers.iter_mut().find(|peer| {
            peer.topology.devices
                .iter()
                .any(|device| device.id == device_id)
        }).ok_or(format!("Cannot find a peer with the device <{}>.", device_id))?;

        peer.topology.devices.retain(|device| device.id != device_id);

        carl.peers.store_peer_descriptor(Clone::clone(peer)).await
            .map_err(|error| format!("Failed to delete peer.\n  {}", error))?;

        Ok(())
    }
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

impl From<Device> for DeviceTable {
    fn from(device: Device) -> Self {
        DeviceTable {
            name: device.name,
            id: device.id,
            location: device.location,
            description: device.description,
            tags: device.tags.join(", "),
        }
    }
}
