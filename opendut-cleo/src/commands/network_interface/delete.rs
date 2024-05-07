use std::collections::HashMap;
use uuid::Uuid;

use opendut_carl_api::carl::CarlClient;
use opendut_types::peer::PeerId;
use opendut_types::util::net::NetworkInterfaceName;

/// Delete a network interface from a peer
#[derive(clap::Parser)]
pub struct DeleteNetworkInterfaceCli {
    ///ID of the peer to delete the network configuration from
    #[arg()]
    peer_id: Uuid,
    ///NetworkConfiguration Interface (at least one)
    #[arg(long("interface"), num_args = 1.., required = true)]
    interfaces: Vec<String>,
}

impl DeleteNetworkInterfaceCli {
    pub async fn execute(self, carl: &mut CarlClient) -> crate::Result<()> {
        let id = PeerId::from(self.peer_id);

        let mut peer = carl.peers
            .get_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to get peer with the id '{}'.\n  {}", id, error))?;

        let network_interface_names = self.interfaces.into_iter()
            .map(NetworkInterfaceName::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        let mut device_interfaces_map: HashMap<NetworkInterfaceName, Vec<String>> = HashMap::new();
        for device in peer.topology.devices.clone() {
            device_interfaces_map.entry(device.interface.name).or_default().push(device.name.to_string());
        };

        for name in network_interface_names {
            if device_interfaces_map.contains_key(&name) {
                Err(format!("Network interface '{}' could not be deleted due to it being used in following devices: {}", name,
                            device_interfaces_map.get(&name).unwrap().join(", ")))?
            }
            peer.network.interfaces.retain(|interface| interface.name.name() != name.name())
        };

        carl.peers.store_peer_descriptor(peer).await
            .map_err(|error| format!("Failed to delete network interfaces for peer.\n  {}", error))?;

        Ok(())
    }
}