use crate::persistence::error::PersistenceError;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;
use opendut_carl_api::carl::peer::StorePeerDescriptorError;
use opendut_types::peer;
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2, PeerNetworkConfiguration};
use opendut_types::peer::state::PeerState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::topology::DeviceDescriptor;
use opendut_types::util::net::NetworkInterfaceName;
use std::ops::Not;
use tracing::{debug, error, info, warn};

pub struct StorePeerDescriptorParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer_descriptor: PeerDescriptor,
    pub options: StorePeerDescriptorOptions
}

#[derive(Clone)]
pub struct StorePeerDescriptorOptions {
    pub bridge_name_default: NetworkInterfaceName
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn store_peer_descriptor(params: StorePeerDescriptorParams) -> Result<PeerId, StorePeerDescriptorError> {

    async fn inner(params: StorePeerDescriptorParams) -> Result<PeerId, StorePeerDescriptorError> {

        let peer_id = params.peer_descriptor.id;
        let peer_name = Clone::clone(&params.peer_descriptor.name);
        let peer_descriptor = params.peer_descriptor;
        let resources_manager = params.resources_manager;

        let is_new_peer = resources_manager.resources_mut(|resources| {
            let old_peer_descriptor = resources.get::<PeerDescriptor>(peer_id)?;
            let is_new_peer = old_peer_descriptor.is_none();

            let (devices_to_add, devices_to_remove): (Vec<DeviceDescriptor>, Vec<DeviceDescriptor>) = if let Some(old_peer_descriptor) = old_peer_descriptor {
                debug!("Updating peer descriptor of '{peer_name}' <{peer_id}>.\n  Old: {old_peer_descriptor:?}\n  New: {peer_descriptor:?}");
                let devices_to_add = peer_descriptor.topology.devices.iter()
                    .filter(|device| old_peer_descriptor.topology.devices.contains(device).not())
                    .cloned()
                    .collect();
                let devices_to_remove = old_peer_descriptor.topology.devices.into_iter()
                    .filter(|device| peer_descriptor.topology.devices.contains(device).not())
                    .collect();
                (devices_to_add, devices_to_remove)
            }
            else {
                debug!("Storing peer descriptor of '{peer_name}' <{peer_id}>.\n  {peer_descriptor:?}");
                (peer_descriptor.topology.devices.to_vec(), Vec::<DeviceDescriptor>::new())
            };

            devices_to_remove.iter().try_for_each(|device| {
                let device_id = device.id;
                let device_name = &device.name;
                resources.remove::<DeviceDescriptor>(device.id)?;
                info!("Removed device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}>.");
                Ok::<_, PersistenceError>(())
            })?;

            devices_to_add.iter().try_for_each(|device| {
                let device_id = device.id;
                let device_name = &device.name;
                resources.insert(device.id, Clone::clone(device))?;
                info!("Added device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}>.");
                Ok::<_, PersistenceError>(())
            })?;

            let peer_network_configuration = {
                let bridge_name = peer_descriptor.clone().network.bridge_name
                    .unwrap_or_else(|| params.options.bridge_name_default);
                PeerNetworkConfiguration {
                    bridge_name,
                }
            };

            let peer_configuration = PeerConfiguration {
                cluster_assignment: None,
                network: peer_network_configuration
            };
            resources.insert(peer_id, peer_configuration)?;


            let peer_configuration2 = {
                let mut peer_configuration2 = PeerConfiguration2::default();
                for executor in Clone::clone(&peer_descriptor.executors).executors.into_iter() {
                    peer_configuration2.insert_executor(executor, peer::configuration::ParameterTarget::Present); //TODO not always Present
                }
                peer_configuration2
            };
            resources.insert(peer_id, peer_configuration2)?; //FIXME don't just insert, but rather update existing values via ID with intelligent logic (in a separate action)


            let peer_state = resources.get::<PeerState>(peer_id)?
                .unwrap_or(PeerState::Down); // If peer is new or no peer was found in the database, we consider it as PeerState::Down.
            resources.insert(peer_id, peer_state)?;
            resources.insert(peer_id, peer_descriptor)?;

            Ok(is_new_peer)
        }).await
        .map_err(|cause: PersistenceError| StorePeerDescriptorError::Internal {
            peer_id,
            peer_name: Clone::clone(&peer_name),
            cause: cause.to_string(),
        })?;

        if is_new_peer {
            if let Vpn::Enabled { vpn_client } = params.vpn {
                debug!("Creating VPN peer <{peer_id}>.");
                vpn_client.create_peer(peer_id)
                    .await
                    .map_err(|cause| StorePeerDescriptorError::Internal {
                        peer_id,
                        peer_name: Clone::clone(&peer_name),
                        cause: cause.to_string()
                    })?; // TODO: When a failure happens, we should rollback changes previously made to resources.
                info!("Successfully created VPN peer <{peer_id}>.");
            } else {
                warn!("VPN disabled. Skipping VPN peer creation!");
            }
        }

        if is_new_peer {
            info!("Successfully stored peer descriptor of '{peer_name}' <{peer_id}>.");
        }
        else {
            info!("Successfully updated peer descriptor of '{peer_name}' <{peer_id}>.");
        }

        Ok(peer_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::actions::peers::testing::{fixture, store_peer_descriptor_options, Fixture};
    use crate::resources::manager::ResourcesManager;
    use googletest::prelude::*;
    use opendut_types::topology::{DeviceDescription, DeviceId, DeviceName, Topology};
    use opendut_types::util::net::NetworkInterfaceId;
    use rstest::rstest;
    use std::sync::Arc;

    #[rstest]
    #[tokio::test]
    async fn should_update_expected_resources_in_memory(fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {
        let resources_manager = ResourcesManager::new_in_memory();
        should_update_expected_resources_implementation(resources_manager, fixture, store_peer_descriptor_options).await
    }

    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[rstest]
    #[tokio::test]
    #[ignore] //FIXME currently broken
    async fn should_update_expected_resources_in_database(fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {
        let db = crate::persistence::database::testing::spawn_and_connect_resources_manager().await?;
        should_update_expected_resources_implementation(db.resources_manager, fixture, store_peer_descriptor_options).await
    }

    async fn should_update_expected_resources_implementation(resources_manager: ResourcesManagerRef, fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {

        store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&resources_manager),
            vpn: Clone::clone(&fixture.vpn),
            peer_descriptor: Clone::clone(&fixture.peer_a_descriptor),
            options: store_peer_descriptor_options.clone(),
        }).await?;

        assert_that!(resources_manager.get::<PeerDescriptor>(fixture.peer_a_id).await?.as_ref(), some(eq(&fixture.peer_a_descriptor)));
        assert_that!(resources_manager.get::<DeviceDescriptor>(fixture.peer_a_device_1).await?.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[0])));
        assert_that!(resources_manager.get::<DeviceDescriptor>(fixture.peer_a_device_2).await?.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[1])));
        assert_that!(resources_manager.get::<PeerState>(fixture.peer_a_id).await?.as_ref(), some(eq(&PeerState::Down)));

        let additional_device_id = DeviceId::random();
        let additional_device = DeviceDescriptor {
            id: additional_device_id,
            name: DeviceName::try_from("PeerA_Device_42")?,
            description: DeviceDescription::try_from("Additional device for peerA").ok(),
            interface: NetworkInterfaceId::random(),
            tags: vec![],
        };

        let changed_descriptor = PeerDescriptor {
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
            options: store_peer_descriptor_options,
        }).await?;

        assert_that!(resources_manager.get::<PeerDescriptor>(fixture.peer_a_id).await?.as_ref(), some(eq(&changed_descriptor)));
        assert_that!(resources_manager.get::<DeviceDescriptor>(fixture.peer_a_device_1).await?.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[0])));
        assert_that!(resources_manager.get::<DeviceDescriptor>(additional_device_id).await?.as_ref(), some(eq(&additional_device)));
        assert_that!(resources_manager.get::<DeviceDescriptor>(fixture.peer_a_device_2).await?.as_ref(), none());
        assert_that!(resources_manager.get::<PeerState>(fixture.peer_a_id).await?.as_ref(), some(eq(&PeerState::Down)));

        Ok(())
    }
}
