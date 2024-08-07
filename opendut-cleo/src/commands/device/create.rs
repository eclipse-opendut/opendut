use uuid::Uuid;
use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::topology::{DeviceDescription, DeviceDescriptor, DeviceId, DeviceName, DeviceTag};
use opendut_types::util::net::NetworkInterfaceName;
use crate::{CreateOutputFormat, DescribeOutputFormat};

/// Create a device
#[derive(clap::Parser)]
pub struct CreateDeviceCli {
    ///ID of the peer to add the device to
    #[arg(long)]
    peer_id: Uuid,
    ///ID of the device to be added or updated
    #[arg(long)]
    device_id: Option<Uuid>,
    ///Name of the device
    #[arg(long)]
    name: Option<String>,
    ///Description of device
    #[arg(long)]
    description: Option<String>,
    ///Interface of device
    #[arg(long)]
    interface: Option<NetworkInterfaceName>,
    /// Tags of device
    #[arg(long("tag"))]
    tags: Option<Vec<String>>,
}

impl CreateDeviceCli {
    #[allow(clippy::too_many_arguments)]
    pub async fn execute(self, carl: &mut CarlClient, output: CreateOutputFormat) -> crate::Result<()> {
        let peer_id = PeerId::from(self.peer_id);
        let device_id = self.device_id.map(DeviceId::from).unwrap_or(DeviceId::random());

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;
        let peer_network_interface_names = peer_descriptor.network.interfaces.iter().map(|peer_interface| {
            peer_interface.name.clone()
        }).collect::<Vec<_>>();
        let maybe_existing_device = peer_descriptor.topology.devices.iter_mut().find(|device| device.id == device_id);
        match maybe_existing_device {
            None => {
                //TODO provide separate `update device` command to not need this custom input handling
                let name = self.name.ok_or(String::from("Cannot create new device because of missing device name."))?;
                let interface_name = self.interface.ok_or(String::from("Cannot create new device because of missing interface name."))?;

                let interface = match peer_descriptor.network.interfaces.iter().find(|descriptor| descriptor.name == interface_name) {
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
                    description: self.description
                        .map(DeviceDescription::try_from)
                        .transpose()
                        .map_err(|error| error.to_string())?,
                    interface: interface.id,
                    tags: self.tags
                        .unwrap_or_default()
                        .into_iter()
                        .map(DeviceTag::try_from)
                        .collect::<Result<_, _>>()
                        .map_err(|error| error.to_string())?,
                };
                peer_descriptor.topology.devices.push(new_device);
            }
            Some(device) => {
                if let Some(name) = self.name {
                    device.name = DeviceName::try_from(name)
                        .map_err(|error| error.to_string())?;
                }
                if let Some(description) = self.description {
                    device.description = DeviceDescription::try_from(description)
                        .map_err(|error| error.to_string())
                        .ok();
                }
                if let Some(interface_name) = self.interface {
                    device.interface = match peer_descriptor.network.interfaces.iter().find(|descriptor| descriptor.name == interface_name) {
                        Some(network_interface_descriptor) => network_interface_descriptor.id,
                        None => {
                            Err(format!("Cannot create new device because interface is not one of the allowed values: {} \nAllowed interfaces are configured on the peer.",
                                        peer_network_interface_names.into_iter().map(|name| name.name()).collect::<Vec<_>>().join(", ")))?
                        },
                    };
                }
                if let Some(tags) = self.tags {
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
