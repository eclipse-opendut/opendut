use std::collections::HashMap;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::util::net::{NetworkInterfaceId, NetworkInterfaceName};

/// Delete a network interface from a peer
#[derive(clap::Parser)]
pub struct DeleteNetworkInterfaceCli {
    ///ID of the peer to delete the network configuration from
    #[arg()]
    peer_id: PeerId,
    ///NetworkConfiguration Interface (at least one)
    #[arg(long("interface"), num_args = 1.., required = true)]
    interfaces: Vec<String>,
}

impl DeleteNetworkInterfaceCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let peer_id = self.peer_id;

        let mut peer = carl.peers
            .get_peer_descriptor(peer_id)
            .await
            .map_err(|error| format!("Failed to get peer with the id '{peer_id}'.\n  {error}"))?;

        let network_interface_names = self.interfaces.into_iter()
            .map(NetworkInterfaceName::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        let network_interfaces = network_interface_names.into_iter()
            .map(|interface_name|
                peer.network.interfaces.iter()
                    .find(|interface| interface.name == interface_name)
                    .cloned()
                    .ok_or_else(|| format!("Peer <{peer_id}> has no network interface with name '{interface_name}'."))
            ).collect::<Result<Vec<_>, _>>()?;

        let mut device_interfaces_map: HashMap<NetworkInterfaceId, Vec<String>> = HashMap::new();
        for device in peer.topology.devices.clone() {
            device_interfaces_map.entry(device.interface).or_default().push(device.name.to_string());
        };

        for interface_to_remove in network_interfaces {
            if device_interfaces_map.contains_key(&interface_to_remove.id) {
                Err(format!(
                    "Network interface '{}' could not be deleted due to it being used in following devices: {}",
                    interface_to_remove.name,
                    device_interfaces_map.get(&interface_to_remove.id).unwrap().join(", ")
                ))?
            }
            peer.network.interfaces.retain(|interface| interface.id != interface_to_remove.id)
        };

        carl.peers.store_peer_descriptor(peer).await
            .map_err(|error| format!("Failed to delete network interfaces for peer.\n  {error}"))?;

        Ok(())
    }
}
