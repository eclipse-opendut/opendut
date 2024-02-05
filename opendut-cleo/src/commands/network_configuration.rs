pub mod create {
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::{PeerId, PeerNetworkInterface};
    use opendut_types::util::net::NetworkInterfaceName;

    use crate::{CreateOutputFormat, DescribeOutputFormat};

    pub async fn execute(
        carl: &mut CarlClient,
        peer_id: Uuid,
        network_configuration: Option<Vec<String>>,
        output: CreateOutputFormat,
    ) -> crate::Result<()> {
        let peer_id = PeerId::from(peer_id);

        let mut peer_descriptor = carl.peers.get_peer_descriptor(peer_id).await
            .map_err(|_| format!("Failed to get peer with ID <{}>.", peer_id))?;

        let peer_interfaces = peer_descriptor.network_configuration.interfaces.clone();

        let new_network_configurations = network_configuration
                .unwrap_or_default()
                .into_iter()
                .map(NetworkInterfaceName::try_from)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|error| error.to_string())?
                .into_iter()
                .map(|interface_name| PeerNetworkInterface { name: interface_name })
                .collect::<Vec<_>>();

        for network_config in new_network_configurations {
            if peer_interfaces.contains(&network_config) {
                Err(format!("Could not create peer network configuration with name '{}' because it already exists", network_config.name))?
            } else {
                peer_descriptor.network_configuration.interfaces.push(network_config);
            }
        }

        carl.peers.store_peer_descriptor(Clone::clone(&peer_descriptor)).await
            .map_err(|error| format!("Failed to update peer <{}>.\n  {}", peer_id, error))?;
        let output_format = DescribeOutputFormat::from(output);
        crate::commands::peer::describe::render_peer_descriptor(peer_descriptor, output_format);

        Ok(())
    }
}

pub mod delete {
    use std::collections::HashMap;
    use uuid::Uuid;

    use opendut_carl_api::carl::CarlClient;
    use opendut_types::peer::PeerId;
    use opendut_types::util::net::NetworkInterfaceName;

    pub async fn execute(carl: &mut CarlClient, id: Uuid, network_configuration: Vec<String>) -> crate::Result<()> {
        let id = PeerId::from(id);

        let mut peer = carl.peers
            .get_peer_descriptor(id)
            .await
            .map_err(|error| format!("Failed to get peer with the id '{}'.\n  {}", id, error))?;

        let network_interface_names = network_configuration.into_iter()
            .map(NetworkInterfaceName::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| error.to_string())?;

        let mut device_interfaces_map: HashMap<NetworkInterfaceName,Vec<String>> = HashMap::new();
        for device in peer.topology.devices.clone() {
            device_interfaces_map.entry(device.interface).or_default().push(device.name.to_string());
        };

        for name in network_interface_names {
            if device_interfaces_map.contains_key(&name) {
                Err(format!("Network interface '{}' could not be deleted due to it being used in following devices: {}", name,
                            device_interfaces_map.get(&name).unwrap().join(", ")))?
            }
            peer.network_configuration.interfaces.retain(|interface| interface.name.name() != name.name())
        };

        carl.peers.store_peer_descriptor(peer).await
            .map_err(|error| format!("Failed to delete network interfaces for peer.\n  {}", error))?;

        Ok(())
    }
}
