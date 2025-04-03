use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;
use crate::settings::vpn::Vpn;
use opendut_types::peer::state::PeerState;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName};
use opendut_types::ShortName;
use tracing::{debug, error, info, warn};

pub struct StorePeerDescriptorParams {
    pub vpn: Vpn,
    pub peer_descriptor: PeerDescriptor,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn store_peer_descriptor(&mut self, params: StorePeerDescriptorParams) -> Result<PeerId, StorePeerDescriptorError> {

        let peer_id = params.peer_descriptor.id;
        let peer_name = Clone::clone(&params.peer_descriptor.name);
        let peer_descriptor = params.peer_descriptor;

        let is_new_peer = self.get::<PeerDescriptor>(peer_id)
            .map_err(|source| StorePeerDescriptorError::Persistence { peer_id, peer_name: peer_name.clone(), source })?
            .is_none();

        if is_new_peer {
            if let Vpn::Enabled { vpn_client } = &params.vpn {
                debug!("Creating VPN peer <{peer_id}>.");
                vpn_client.create_peer(peer_id).await
                    .map_err(|source| StorePeerDescriptorError::VpnClient { peer_id, peer_name: Clone::clone(&peer_name), source })?;
                info!("Successfully created VPN peer <{peer_id}>.");
            } else {
                warn!("VPN disabled. Skipping VPN peer creation!");
            }
        }

        let persistence_result = self.insert(peer_id, peer_descriptor)
            .map(|()| peer_id)
            .map_err(|source| StorePeerDescriptorError::Persistence { peer_id, peer_name: peer_name.clone(), source });

        if persistence_result.is_err() && is_new_peer { //undo creating peer in VPN Management server when storing in database fails
            if let Vpn::Enabled { vpn_client } = params.vpn {
                debug!("Deleting previously created VPN peer <{peer_id}> due to persistence error.");
                let result = vpn_client.delete_peer(peer_id).await;
                match result {
                    Ok(()) => info!("Successfully deleted previously created VPN peer <{peer_id}>."),
                    Err(cause) => error!("Failed to delete previously created VPN peer <{peer_id}>: {cause}\n  Cannot recover automatically. Please remove the peer from the VPN management server manually."),
                }
            }
        }

        if is_new_peer {
            info!("Successfully stored peer descriptor of '{peer_name}' <{peer_id}>.");
        } else {
            info!("Successfully updated peer descriptor of '{peer_name}' <{peer_id}>.");
        }

        persistence_result
    }
}

#[derive(thiserror::Error, Debug)]
pub enum StorePeerDescriptorError {
    #[error("Peer '{peer_name}' <{peer_id}> cannot be updated in state '{}'! A peer can be updated when: {}", actual_state.short_name(), PeerState::short_names_joined(required_states))]
    IllegalPeerState {
        peer_id: PeerId,
        peer_name: PeerName,
        actual_state: PeerState,
        required_states: Vec<PeerState>,
    },
    #[error("Error when accessing persistence while creating peer '{peer_name}' <{peer_id}>")]
    Persistence {
        peer_id: PeerId,
        peer_name: PeerName,
        #[source] source: PersistenceError,
    },
    #[error("Error when creating peer in VPN management while store peer descriptor for peer '{peer_name}' <{peer_id}>")]
    VpnClient {
        peer_id: PeerId,
        peer_name: PeerName,
        #[source] source: opendut_vpn::CreatePeerError,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::testing::PeerFixture;
    use crate::resource::manager::{ResourceManager, ResourceManagerRef};
    use googletest::prelude::*;
    use opendut_types::peer::PeerNetworkDescriptor;
    use opendut_types::topology::DeviceDescriptor;
    use opendut_types::topology::{DeviceDescription, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    #[tokio::test]
    async fn should_update_expected_resources_in_memory() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        should_update_expected_resources_implementation(resource_manager).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn should_update_expected_resources_in_database() -> anyhow::Result<()> {
        let db = crate::resource::persistence::testing::spawn_and_connect_resource_manager().await?;
        should_update_expected_resources_implementation(db.resource_manager).await
    }

    async fn should_update_expected_resources_implementation(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
        let peer = PeerFixture::new();

        resource_manager.resources_mut(async |resources|
            resources.store_peer_descriptor(StorePeerDescriptorParams {
                vpn: Vpn::Disabled,
                peer_descriptor: Clone::clone(&peer.descriptor),
            }).await
        ).await??;

        assert_that!(resource_manager.get::<PeerDescriptor>(peer.id).await?.as_ref(), some(eq(&peer.descriptor)));
        // TODO: what about PeerState?

        let additional_network_interface = NetworkInterfaceDescriptor {
            id: NetworkInterfaceId::random(),
            name: NetworkInterfaceName::try_from("eth2")?,
            configuration: NetworkInterfaceConfiguration::Ethernet,
        };

        let additional_device = DeviceDescriptor {
            id: DeviceId::random(),
            name: DeviceName::try_from("PeerA_Device_42")?,
            description: DeviceDescription::try_from("Additional device for peerA").ok(),
            interface: additional_network_interface.id,
            tags: vec![],
        };

        let changed_descriptor = PeerDescriptor {
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    // 1 interface removed
                    additional_network_interface,
                ],
                ..Clone::clone(&peer.descriptor.network)
            },
            topology: Topology {
                devices: vec![
                    // 1 device removed
                    Clone::clone(&additional_device),
                ]
            },
            ..Clone::clone(&peer.descriptor)
        };

        resource_manager.resources_mut(async |resources|
            resources.store_peer_descriptor(StorePeerDescriptorParams {
                vpn: Vpn::Disabled,
                peer_descriptor: Clone::clone(&changed_descriptor),
            }).await
        ).await??;

        assert_that!(resource_manager.get::<PeerDescriptor>(peer.id).await?.as_ref(), some(eq(&changed_descriptor)));
        // TODO: what about PeerState?

        Ok(())
    }
}
