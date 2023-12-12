use std::ops::Not;
use std::sync::Arc;

use url::Url;

pub use opendut_carl_api::carl::peer::{
    CreatePeerError,
    DeletePeerError,
    ListDevicesError,
    ListPeersError,
    RegisterDevicesError,
    UnregisterDevicesError
};
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerSetup};
use opendut_types::topology::{Device, DeviceId};
use opendut_types::vpn::VpnPeerConfig;
use opendut_util::ErrorOr;
use opendut_util::logging::LogError;

use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

pub struct CreatePeerParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer: PeerDescriptor,
}

pub async fn create_peer(params: CreatePeerParams) -> Result<PeerId, CreatePeerError> {

    async fn inner(params: CreatePeerParams) -> Result<PeerId, CreatePeerError> {

        let peer_id = params.peer.id;
        let peer_name = Clone::clone(&params.peer.name);
        let resources_manager = params.resources_manager;

        log::debug!("Creating peer '{peer_name}' <{peer_id}>.");

        if params.peer.topology.devices.is_empty() {
            log::warn!("Peer '{peer_name}' <{peer_id}> will be created without any devices!")
        }
        else {
            register_devices(Arc::clone(&resources_manager), &params.peer)
                .await
                .map_err(|error| CreatePeerError::IllegalDevices {
                    peer_id,
                    peer_name: Clone::clone(&peer_name),
                    error
                })?;
        }

        resources_manager.resources_mut(|resources| {
            if let Some(other_peer) = resources.get::<PeerDescriptor>(peer_id) {
                Err(CreatePeerError::PeerAlreadyExists {
                    actual_id: peer_id,
                    actual_name: Clone::clone(&peer_name),
                    other_id: other_peer.id,
                    other_name: Clone::clone(&other_peer.name),
                })
            } else {
                resources.insert(peer_id, params.peer);
                Ok(())
            }
        }).await?; // TODO: When a failure happen we should rollback previously made changes made to resources.

        if let Vpn::Enabled { vpn_client } = params.vpn {
            log::debug!("Creating vpn peer <{peer_id}>.");
            vpn_client.create_peer(peer_id)
                .await
                .map_err(|cause| CreatePeerError::Internal {
                    peer_id,
                    peer_name: Clone::clone(&peer_name),
                    cause: cause.to_string()
                })?; // TODO: When a failure happen we should rollback previously made changes made to resources.
            log::info!("Successfully created vpn peer <{peer_id}>.");
        } else {
            log::warn!("VPN disabled. Skipping vpn peer creation!");
        }

        log::info!("Successfully created peer '{peer_name}' <{peer_id}>.");

        Ok(peer_id)
    }

    inner(params).await
        .log_err()
}

pub struct DeletePeerParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer: PeerId,
}

pub async fn delete_peer(params: DeletePeerParams) -> Result<PeerDescriptor, DeletePeerError> {

    async fn inner(params: DeletePeerParams) -> Result<PeerDescriptor, DeletePeerError> {

        let peer_id = params.peer;
        let resources_manager = params.resources_manager;

        log::debug!("Deleting peer <{peer_id}>.");

        let peer = resources_manager.resources_mut(|resources| {
            resources.remove::<PeerDescriptor>(peer_id)
                .ok_or_else(|| DeletePeerError::PeerNotFound { peer_id })
        }).await?;

        let peer_name = Clone::clone(&peer.name);

        unregister_devices(Arc::clone(&resources_manager), &peer)
            .await
            .map_err(|error| DeletePeerError::IllegalDevices {
                peer_id,
                peer_name: Clone::clone(&peer_name),
                error
            })
            .err_logged(); // TODO: Decide how to handle the abort?

        if let Vpn::Enabled { vpn_client } = params.vpn {
            log::debug!("Deleting vpn peer <{peer_id}>.");
            vpn_client.delete_peer(peer_id)
                .await
                .map_err(|cause| DeletePeerError::Internal {
                    peer_id,
                    peer_name: Clone::clone(&peer_name),
                    cause: cause.to_string()
                })?;
            log::info!("Successfully deleted vpn peer <{peer_id}>.");
        } else {
            log::warn!("VPN disabled. Skipping vpn peer deletion!");
        }

        log::info!("Successfully deleted peer '{peer_name}' <{peer_id}>.");

        Ok(peer)
    }

    inner(params).await
        .log_err()
}

pub struct ListPeerParams {
    pub resources_manager: ResourcesManagerRef,
}

pub async fn list_peer(params: ListPeerParams) -> Result<Vec<PeerDescriptor>, ListPeersError> {

    async fn inner(params: ListPeerParams) -> Result<Vec<PeerDescriptor>, ListPeersError> {

        let resources_manager = params.resources_manager;

        log::debug!("Querying all peers.");

        let peers = resources_manager.resources(|resources| {
            resources.iter::<PeerDescriptor>()
                .cloned()
                .collect::<Vec<PeerDescriptor>>()
        }).await;

        log::info!("Successfully queried all peers.");

        Ok(peers)
    }

    inner(params).await
        .log_err()
}

pub struct ListDevicesParams {
    pub resources_manager: ResourcesManagerRef,
}

pub async fn list_devices(params: ListDevicesParams) -> Result<Vec<Device>, ListDevicesError> {

    async fn inner(params: ListDevicesParams) -> Result<Vec<Device>, ListDevicesError> {

        let resources_manager = params.resources_manager;

        log::debug!("Querying all devices.");

        let devices = resources_manager.resources(|resource| {
            resource.iter::<Device>().cloned().collect::<Vec<_>>()
        }).await;

        log::info!("Successfully queried all peers.");

        Ok(devices)
    }

    inner(params).await
        .log_err()
}

pub struct CreatePeerSetupParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer: PeerId,
    pub carl_url: Url,
    pub vpn: Vpn,
}

#[derive(thiserror::Error, Debug)]
pub enum CreatePeerSetupError {
    #[error("A PeerSetup for peer <{0}> could not be create, because a peer with that id does not exist!")]
    PeerNotFound(PeerId),
    #[error("An internal error occurred while creating a PeerSetup for peer '{peer_name}' <{peer_id}>:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

pub async fn create_peer_setup(params: CreatePeerSetupParams) -> Result<PeerSetup, CreatePeerSetupError> {

    async fn inner(params: CreatePeerSetupParams) -> Result<PeerSetup, CreatePeerSetupError> {

        let peer_id = params.peer;

        log::debug!("Creating PeerSetup for peer <{peer_id}>");

        let peer_descriptor = params.resources_manager.get::<PeerDescriptor>(peer_id).await
            .ok_or(CreatePeerSetupError::PeerNotFound(peer_id))?;

        let peer_name = peer_descriptor.name;

        let vpn_config = if let Vpn::Enabled { vpn_client } = &params.vpn {
            log::debug!("Retrieving VPN configuration for peer <{peer_id}>.");
            let vpn_config = vpn_client.get_or_create_configuration(params.peer).await
                .map_err(|cause| CreatePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
            log::info!("Successfully retreived vpn configuration for peer <{peer_id}>.");
            vpn_config
        }
        else {
            log::warn!("VPN is disabled. PeerSetup for peer '{peer_name}' <{peer_id}> will not contain any vpn information!");
            VpnPeerConfig::Disabled
        };

        log::debug!("Successfully created peer setup for peer '{peer_name}' <{peer_id}>.");

        Ok(PeerSetup {
            id: peer_id,
            carl: params.carl_url,
            vpn: vpn_config
        })
    }

    inner(params).await
        .log_err()
}

async fn register_devices(resources_manager: ResourcesManagerRef, peer: &PeerDescriptor) -> Result<(), RegisterDevicesError> {
    let peer_id = peer.id;
    let peer_name = Clone::clone(&peer.name);
    let device_count = peer.topology.devices.len();
    log::debug!("Registering devices of peer '{peer_name}' <{peer_id}>.");
    resources_manager.resources_mut(|resources| {
        if let Some(error) = peer.topology.devices.iter().find_map(|device| {
            resources.contains::<Device>(device.id)
                .then_some(RegisterDevicesError::DeviceAlreadyExists { // TODO: Look up other device and peer (owner) to create a more meaningful error message
                    device_id: device.id,
                })
        }) {
            Err(error)
        } else {
            peer.topology.devices.iter()
                .cloned()
                .enumerate()
                .for_each(|(index, device)| {
                    let device_id = device.id;
                    let device_name = Clone::clone(&device.name);
                    resources.insert(device_id, device);
                    log::debug!("Registered device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}> ({index}/{device_count}).", index=index + 1)
                });
            log::info!("Successfully registered {device_count} devices of peer '{peer_name}' <{peer_id}>.");
            Ok(())
        }
    }).await
}

async fn unregister_devices(resources_manager: ResourcesManagerRef, peer: &PeerDescriptor) -> Result<(), UnregisterDevicesError> {
    resources_manager.resources_mut(|resources| {
        peer.topology.devices.iter()
            .find(|device| resources.contains::<Device>(device.id).not())
            .map(|device| UnregisterDevicesError::DeviceNotFound { device_id: device.id })
            .err_or(())?;
        peer.topology.devices.iter().for_each(|device| {
            resources.remove::<Device>(device.id);
        });
        Ok(())
    }).await
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use rstest::*;

    use opendut_types::peer::PeerName;
    use opendut_types::topology::{InterfaceName, Topology};

    use crate::resources::manager::ResourcesManager;

    use super::*;

    mod create_peer {
        use googletest::prelude::*;
        use rstest::*;

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_add_expected_resources(fixture: Fixture) -> anyhow::Result<()> {

            let resources_manager = fixture.resources_manager;

            create_peer(CreatePeerParams {
                resources_manager: Arc::clone(&resources_manager),
                vpn: fixture.vpn,
                peer: Clone::clone(&fixture.peer_a_descriptor),
            }).await?;

            assert_that!(resources_manager.get(fixture.peer_a_id).await.as_ref(), some(eq(&fixture.peer_a_descriptor)));
            assert_that!(resources_manager.get(fixture.peer_a_device_1).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[0])));
            assert_that!(resources_manager.get(fixture.peer_a_device_2).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[1])));

            Ok(())
        }
    }

    mod list_devices {
        use std::sync::Arc;

        use googletest::prelude::*;
        use rstest::*;

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_return_all_devices(fixture: Fixture) -> anyhow::Result<()> {
            let resources_manager = fixture.resources_manager;

            let devices = list_devices(ListDevicesParams {
                resources_manager: Arc::clone(&resources_manager),
            }).await;

            assert_that!(devices, ok(empty()));

            register_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await?;

            let devices = list_devices(ListDevicesParams {
                resources_manager: Arc::clone(&resources_manager),
            }).await;

            assert_that!(devices, ok(unordered_elements_are![
                eq_deref_of(&fixture.peer_a_descriptor.topology.devices[0]),
                eq_deref_of(&fixture.peer_a_descriptor.topology.devices[1]),
            ]));

            Ok(())
        }
    }

    mod register_devices {
        use std::sync::Arc;

        use googletest::prelude::*;
        use rstest::*;

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_add_expected_resources(fixture: Fixture) -> anyhow::Result<()> {
            let resources_manager = fixture.resources_manager;

            register_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await?;

            assert_that!(resources_manager.get(fixture.peer_a_device_1).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[0])));
            assert_that!(resources_manager.get(fixture.peer_a_device_2).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[1])));

            Ok(())
        }

        #[rstest]
        #[tokio::test]
        async fn should_fail_if_a_device_with_the_same_id_already_exists(fixture: Fixture) -> anyhow::Result<()> {
            let resources_manager = fixture.resources_manager;

            register_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await?;

            assert_that!(register_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await,
                err(matches_pattern!(RegisterDevicesError::DeviceAlreadyExists {
                    device_id: eq(fixture.peer_a_descriptor.topology.devices[0].id),
                })));

            Ok(())
        }
    }

    mod unregister_devices {
        use std::sync::Arc;

        use googletest::prelude::*;
        use rstest::*;

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_remove_all_related_resources(fixture: Fixture) -> anyhow::Result<()> {
            let resources_manager = fixture.resources_manager;

            register_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await?;

            assert_that!(resources_manager.is_not_empty().await, eq(true));

            unregister_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await?;

            assert_that!(resources_manager.is_empty().await, eq(true));

            Ok(())
        }

        #[rstest]
        #[tokio::test]
        async fn should_fail_when_a_device_to_remove_does_not_exist(fixture: Fixture) -> anyhow::Result<()> {
            let resources_manager = fixture.resources_manager;

            assert_that!(unregister_devices(Arc::clone(&resources_manager), &fixture.peer_a_descriptor).await,
                err(anything()));

            Ok(())
        }
    }

    struct Fixture {
        resources_manager: ResourcesManagerRef,
        vpn: Vpn,
        peer_a_id: PeerId,
        peer_a_descriptor: PeerDescriptor,
        peer_a_device_1: DeviceId,
        peer_a_device_2: DeviceId,
    }

    #[fixture]
    fn fixture() -> Fixture {
        let peer_a_id = PeerId::random();
        let peer_a_device_1 = DeviceId::random();
        let peer_a_device_2 = DeviceId::random();
        let peer_a_descriptor = PeerDescriptor {
            id: peer_a_id,
            name: PeerName::try_from("PeerA").unwrap(),
            topology: Topology {
                devices: vec![
                    Device {
                        id: peer_a_device_1,
                        name: String::from("PeerA Device 1"),
                        description: String::from("Huii"),
                        location: String::from("Ulm"),
                        interface: InterfaceName::try_from("eth0").unwrap(),
                        tags: vec![],
                    },
                    Device {
                        id: peer_a_device_2,
                        name: String::from("PeerA Device 2"),
                        description: String::from("Huii"),
                        location: String::from("Stuttgart"),
                        interface: InterfaceName::try_from("eth1").unwrap(),
                        tags: vec![],
                    }
                ]
            },
        };
        Fixture {
            resources_manager: Arc::new(ResourcesManager::new()),
            vpn: Vpn::Disabled,
            peer_a_id,
            peer_a_descriptor,
            peer_a_device_1,
            peer_a_device_2,
        }
    }
}
