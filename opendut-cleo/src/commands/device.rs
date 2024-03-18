use cli_table::{Table, WithTitle};
use serde::Serialize;

use opendut_carl_api::carl::CarlClient;
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
    use opendut_types::peer::PeerId;
    use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag};
    use opendut_types::util::net::NetworkInterfaceName;
    use crate::{CreateOutputFormat, DescribeOutputFormat};

    #[allow(clippy::too_many_arguments)]
    pub async fn execute(
        carl: &mut CarlClient,
        peer_id: Uuid,
        device_id: Option<Uuid>,
        name: Option<String>,
        description: Option<String>,
        interface_name: Option<NetworkInterfaceName>,
        tags: Option<Vec<String>>,
        output: CreateOutputFormat,
    ) -> crate::Result<()> {
        let peer_id = PeerId::from(peer_id);
        let device_id = device_id.map(DeviceId::from).unwrap_or(DeviceId::random());

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;
        let peer_network_interface_names = peer_descriptor.network_configuration.interfaces.iter().map(|peer_interface| {
            peer_interface.name.clone()
        }).collect::<Vec<_>>();
        let maybe_existing_device = peer_descriptor.topology.devices.iter_mut().find(|device| device.id == device_id) ;
        match maybe_existing_device {
            None => {
                //TODO provide separate `update device` command to not need this custom input handling
                let name = name.ok_or(String::from("Cannot create new device because of missing device name."))?;
                let interface_name = interface_name.ok_or(String::from("Cannot create new device because of missing interface name."))?;

                let interface = match peer_descriptor.network_configuration.interfaces.iter().find(|descriptor| descriptor.name == interface_name) {
                    Some(network_interface_descriptor) => network_interface_descriptor.clone(),
                    None => {
                        Err(format!("Cannot create new device because interface is not one of the allowed values: {} \nAllowed interfaces are configured on the peer.",
                                peer_network_interface_names.into_iter().map(|name| name.name()).collect::<Vec<_>>().join(", ")))?
                    },
                };

                let new_device = DeviceDescriptor {
                    id: device_id,
                    name: DeviceName::try_from(name)
                        .map_err(|error| error.to_string())?,
                    description: description
                        .map(DeviceDescription::try_from)
                        .transpose()
                        .map_err(|error| error.to_string())?,
                    interface,
                    tags: tags
                        .unwrap_or_default()
                        .into_iter()
                        .map(DeviceTag::try_from)
                        .collect::<Result<_, _>>()
                        .map_err(|error| error.to_string())?,
                };
                peer_descriptor.topology.devices.push(new_device);
            }
            Some(device) => {
                if let Some(name) = name {
                    device.name = DeviceName::try_from(name)
                        .map_err(|error| error.to_string())?;
                }
                if let Some(description) = description {
                    device.description = DeviceDescription::try_from(description)
                        .map_err(|error| error.to_string())
                        .ok();
                }
                if let Some(interface_name) = interface_name {
                    device.interface = match peer_descriptor.network_configuration.interfaces.iter().find(|descriptor| descriptor.name == interface_name) {
                        Some(network_interface_descriptor) => network_interface_descriptor.clone(),
                        None => {
                            Err(format!("Cannot create new device because interface is not one of the allowed values: {} \nAllowed interfaces are configured on the peer.",
                                    peer_network_interface_names.into_iter().map(|name| name.name()).collect::<Vec<_>>().join(", ")))?
                        },
                    };
                }
                if let Some(tags) = tags {
                    device.tags = tags
                        .into_iter()
                        .map(DeviceTag::try_from)
                        .collect::<Result<_, _>>()
                        .map_err(|error| error.to_string())?;
                }
            }
        }
        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        let output_format = DescribeOutputFormat::from(output);
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);
        
        Ok(())
    }
}

pub mod describe {
    use indoc::indoc;
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::topology::{DeviceDescription, DeviceId};

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
