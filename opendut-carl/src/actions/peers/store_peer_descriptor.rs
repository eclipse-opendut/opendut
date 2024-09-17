use crate::persistence::error::PersistenceError;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;
use opendut_carl_api::carl::peer::StorePeerDescriptorError;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info, warn};

pub struct StorePeerDescriptorParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer_descriptor: PeerDescriptor,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn store_peer_descriptor(params: StorePeerDescriptorParams) -> Result<PeerId, StorePeerDescriptorError> {

    async fn inner(params: StorePeerDescriptorParams) -> Result<PeerId, StorePeerDescriptorError> {

        let peer_id = params.peer_descriptor.id;
        let peer_name = Clone::clone(&params.peer_descriptor.name);
        let peer_descriptor = params.peer_descriptor;
        let resources_manager = params.resources_manager;

        let is_new_peer = resources_manager.get::<PeerDescriptor>(peer_id).await
            .map_err(|cause| StorePeerDescriptorError::Internal { peer_id, peer_name: peer_name.clone(), cause: cause.to_string() })?
            .is_none();

        if is_new_peer {
            if let Vpn::Enabled { vpn_client } = &params.vpn {
                debug!("Creating VPN peer <{peer_id}>.");
                vpn_client.create_peer(peer_id).await
                    .map_err(|cause| StorePeerDescriptorError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
                info!("Successfully created VPN peer <{peer_id}>.");
            } else {
                warn!("VPN disabled. Skipping VPN peer creation!");
            }
        }

        let persistence_result = resources_manager.insert(peer_id, peer_descriptor).await
            .map(|()| peer_id)
            .map_err(|cause: PersistenceError| StorePeerDescriptorError::Internal { peer_id, peer_name: peer_name.clone(), cause: cause.to_string() });

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

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::peers::testing::{fixture, Fixture};
    use crate::resources::manager::ResourcesManager;
    use googletest::prelude::*;
    use opendut_types::peer::state::PeerState;
    use opendut_types::peer::PeerNetworkDescriptor;
    use opendut_types::topology::DeviceDescriptor;
    use opendut_types::topology::{DeviceDescription, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use rstest::rstest;
    use std::sync::Arc;

    #[rstest]
    #[tokio::test]
    async fn should_update_expected_resources_in_memory(fixture: Fixture) -> anyhow::Result<()> {
        let resources_manager = ResourcesManager::new_in_memory();
        should_update_expected_resources_implementation(resources_manager, fixture).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[rstest]
    #[tokio::test]
    async fn should_update_expected_resources_in_database(fixture: Fixture) -> anyhow::Result<()> {
        let db = crate::persistence::database::testing::spawn_and_connect_resources_manager().await?;
        should_update_expected_resources_implementation(db.resources_manager, fixture).await
    }

    async fn should_update_expected_resources_implementation(resources_manager: ResourcesManagerRef, fixture: Fixture) -> anyhow::Result<()> {

        store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Clone::clone(&fixture.vpn),
            peer_descriptor: Clone::clone(&fixture.peer_a_descriptor),
        }).await?;

        assert_that!(resources_manager.get::<PeerDescriptor>(fixture.peer_a_id).await?.as_ref(), some(eq(&fixture.peer_a_descriptor)));
        assert_that!(resources_manager.get::<PeerState>(fixture.peer_a_id).await?.as_ref(), none());


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
                    Clone::clone(&fixture.peer_a_descriptor.network.interfaces[0]),
                    additional_network_interface,
                ],
                ..Clone::clone(&fixture.peer_a_descriptor.network)
            },
            topology: Topology {
                devices: vec![
                    Clone::clone(&fixture.peer_a_descriptor.topology.devices[0]),
                    Clone::clone(&additional_device),
                ]
            },
            ..Clone::clone(&fixture.peer_a_descriptor)
        };

        store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Clone::clone(&fixture.vpn),
            peer_descriptor: Clone::clone(&changed_descriptor),
        }).await?;

        assert_that!(resources_manager.get::<PeerDescriptor>(fixture.peer_a_id).await?.as_ref(), some(eq(&changed_descriptor)));
        assert_that!(resources_manager.get::<PeerState>(fixture.peer_a_id).await?.as_ref(), none());

        Ok(())
    }
}
