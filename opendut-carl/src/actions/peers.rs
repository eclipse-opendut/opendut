use std::ops::Not;
use std::sync::Arc;

use pem::Pem;
use tracing::Span;
use url::Url;

pub use opendut_carl_api::carl::peer::{
    DeletePeerDescriptorError,
    IllegalDevicesError,
    ListDevicesError,
    ListPeerDescriptorsError,
    StorePeerDescriptorError,
};
use opendut_carl_api::proto::services::peer_messaging_broker::{ApplyPeerConfiguration, downstream};
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerSetup};
use opendut_types::peer::configuration::PeerConfiguration;
use opendut_types::proto;
use opendut_types::topology::{DeviceDescriptor, DeviceId};
use opendut_types::util::net::{AuthConfig, Certificate, ClientCredentials};
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::ErrorOr;
use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerRef};
use crate::peer::oidc_client_manager::{OAuthClientCredentials, OpenIdConnectClientManager};

use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

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

        let is_new_peer = resources_manager.resources_mut(|resources| {

            let old_peer_descriptor = resources.get::<PeerDescriptor>(peer_id);
            let is_new_peer = old_peer_descriptor.is_none();

            let (devices_to_add, devices_to_remove): (Vec<DeviceDescriptor>, Vec<DeviceDescriptor>) = if let Some(old_peer_descriptor) = old_peer_descriptor {
                log::debug!("Updating peer descriptor of '{peer_name}' <{peer_id}>.\n  Old: {old_peer_descriptor:?}\n  New: {peer_descriptor:?}");
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
                log::debug!("Storing peer descriptor of '{peer_name}' <{peer_id}>.\n  {peer_descriptor:?}");
                (peer_descriptor.topology.devices.to_vec(), Vec::<DeviceDescriptor>::new())
            };

            devices_to_remove.iter().for_each(|device| {
                let device_id = device.id;
                let device_name = &device.name;
                resources.remove(device.id);
                log::info!("Removed device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}>.");
            });

            devices_to_add.iter().for_each(|device| {
                let device_id = device.id;
                let device_name = &device.name;
                resources.insert(device.id, Clone::clone(device));
                log::info!("Added device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}>.");
            });

            let peer_configuration = PeerConfiguration {
                executors: Clone::clone(&peer_descriptor.executors),
                cluster_assignment: None,
            };
            resources.insert(peer_id, peer_configuration);

            resources.insert(peer_id, peer_descriptor);

            is_new_peer
        }).await;

        if is_new_peer {
            if let Vpn::Enabled { vpn_client } = params.vpn {
                log::debug!("Creating VPN peer <{peer_id}>.");
                vpn_client.create_peer(peer_id)
                    .await
                    .map_err(|cause| StorePeerDescriptorError::Internal {
                        peer_id,
                        peer_name: Clone::clone(&peer_name),
                        cause: cause.to_string()
                    })?; // TODO: When a failure happens, we should rollback changes previously made to resources.
                log::info!("Successfully created VPN peer <{peer_id}>.");
            } else {
                log::warn!("VPN disabled. Skipping VPN peer creation!");
            }
        }

        if is_new_peer {
            log::info!("Successfully stored peer descriptor of '{peer_name}' <{peer_id}>.");
        }
        else {
            log::info!("Successfully updated peer descriptor of '{peer_name}' <{peer_id}>.");
        }

        Ok(peer_id)
    }

    inner(params).await
        .inspect_err(|err| log::error!("{err}"))
}

pub struct DeletePeerDescriptorParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer: PeerId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_peer_descriptor(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

    async fn inner(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

        let peer_id = params.peer;
        let resources_manager = params.resources_manager;

        log::debug!("Deleting peer descriptor of peer <{peer_id}>.");

        let peer_descriptor = resources_manager.resources_mut(|resources| {

            let peer_descriptor = resources.remove::<PeerDescriptor>(peer_id)
                .ok_or_else(|| DeletePeerDescriptorError::PeerNotFound { peer_id })?;

            let peer_name = &peer_descriptor.name;

            peer_descriptor.topology.devices.iter().for_each(|device| {
                let device_id = device.id;
                let device_name = &device.name;
                resources.remove(device_id);
                log::debug!("Deleted device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}>.");
            });

            Ok(peer_descriptor)
        }).await?;

        let peer_name = &peer_descriptor.name;

        if let Vpn::Enabled { vpn_client } = params.vpn {
            log::debug!("Deleting vpn peer <{peer_id}>.");
            vpn_client.delete_peer(peer_id)
                .await
                .map_err(|cause| DeletePeerDescriptorError::Internal {
                    peer_id,
                    peer_name: Clone::clone(peer_name),
                    cause: cause.to_string()
                })?;
            log::info!("Successfully deleted VPN peer <{peer_id}>.");
        } else {
            log::warn!("VPN disabled. Skipping VPN peer deletion!");
        }

        log::info!("Successfully deleted peer descriptor of '{peer_name}' <{peer_id}>.");

        Ok(peer_descriptor)
    }

    inner(params).await
        .inspect_err(|err| log::error!("{err}"))
}

pub struct ListPeerDescriptorsParams {
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_peer_descriptors(params: ListPeerDescriptorsParams) -> Result<Vec<PeerDescriptor>, ListPeerDescriptorsError> {

    async fn inner(params: ListPeerDescriptorsParams) -> Result<Vec<PeerDescriptor>, ListPeerDescriptorsError> {

        let resources_manager = params.resources_manager;

        log::debug!("Querying all peer descriptors.");

        let peers = resources_manager.resources(|resources| {
            resources.iter::<PeerDescriptor>()
                .cloned()
                .collect::<Vec<PeerDescriptor>>()
        }).await;

        log::info!("Successfully queried all peer descriptors.");

        Ok(peers)
    }

    inner(params).await
        .inspect_err(|err| log::error!("{err}"))
}

pub struct ListDevicesParams {
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_devices(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

    async fn inner(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

        let resources_manager = params.resources_manager;

        log::debug!("Querying all devices.");

        let devices = resources_manager.resources(|resource| {
            resource.iter::<DeviceDescriptor>().cloned().collect::<Vec<_>>()
        }).await;

        log::info!("Successfully queried all peers.");

        Ok(devices)
    }

    inner(params).await
        .inspect_err(|err| log::error!("{err}"))
}

pub struct GeneratePeerSetupParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer: PeerId,
    pub carl_url: Url,
    pub ca: Pem,
    pub vpn: Vpn,
    pub oidc_client_manager: Option<OpenIdConnectClientManager>,
}

#[derive(thiserror::Error, Debug)]
pub enum GeneratePeerSetupError {
    #[error("A PeerSetup for peer <{0}> could not be created, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("An internal error occurred while creating a PeerSetup for peer '{peer_name}' <{peer_id}>:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn generate_peer_setup(params: GeneratePeerSetupParams) -> Result<PeerSetup, GeneratePeerSetupError> {

    async fn inner(params: GeneratePeerSetupParams) -> Result<PeerSetup, GeneratePeerSetupError> {

        let peer_id = params.peer;

        log::debug!("Generating PeerSetup for peer <{peer_id}>");

        let peer_descriptor = params.resources_manager.get::<PeerDescriptor>(peer_id).await
            .ok_or(GeneratePeerSetupError::PeerNotFound(peer_id))?;

        let peer_name = peer_descriptor.name;

        let vpn_config = if let Vpn::Enabled { vpn_client } = &params.vpn {
            log::debug!("Retrieving VPN configuration for peer <{peer_id}>.");
            let vpn_config = vpn_client.generate_vpn_peer_configuration(params.peer).await
                .map_err(|cause| GeneratePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
            log::info!("Successfully retrieved vpn configuration for peer <{peer_id}>.");
            vpn_config
        }
        else {
            log::warn!("VPN is disabled. PeerSetup for peer '{peer_name}' <{peer_id}> will not contain any VPN information!");
            VpnPeerConfiguration::Disabled
        };

        let auth_config = match params.oidc_client_manager {
            None => {
                AuthConfig::disabled()
            }
            Some(oidc_client_manager) => {
                log::debug!("Generating OIDC client for peer '{peer_name}' <{peer_id}>.");
                let issuer_url = oidc_client_manager.issuer_remote_url.clone();
                let client_credentials = ClientCredentials::from(oidc_client_manager.register_new_client()
                    .await
                    .map_err(|cause| GeneratePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?
                );
                log::debug!("Successfully generated peer setup for peer '{peer_name}' <{peer_id}>. OIDC client_id='{}'.", client_credentials.client_id.clone().value());
                AuthConfig::from_credentials(issuer_url, client_credentials)
            }
        };

        Ok(PeerSetup {
            id: peer_id,
            carl: params.carl_url,
            ca: Certificate(params.ca),
            auth_config,
            vpn: vpn_config,
        })
    }

    inner(params).await
        .inspect_err(|err| log::error!("{err}"))
}


pub struct AssignClusterParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer_messaging_broker: PeerMessagingBrokerRef,
    pub peer_id: PeerId,
    pub cluster_assignment: ClusterAssignment,
}

#[derive(thiserror::Error, Debug)]
pub enum AssignClusterError {
    #[error("Assigning cluster for peer <{0}> failed, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("Sending PeerConfiguration with ClusterAssignment to peer <{peer_id}> failed: {cause}")]
    SendingToPeerFailed { peer_id: PeerId, cause: String },
}

pub async fn assign_cluster(params: AssignClusterParams) -> Result<(), AssignClusterError> {

    let peer_id = params.peer_id;

    let peer_configuration = params.resources_manager.resources_mut(|resources| {
        resources.get::<PeerConfiguration>(peer_id)
            .ok_or(AssignClusterError::PeerNotFound(peer_id))
            .map(|peer_configuration| {
                let peer_configuration = PeerConfiguration {
                    cluster_assignment: Some(params.cluster_assignment),
                    ..peer_configuration
                };
                resources.insert(peer_id, Clone::clone(&peer_configuration));
                peer_configuration
            })
    }).await?;

    params.peer_messaging_broker.send_to_peer(
        peer_id,
        downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
            configuration: Some(peer_configuration.into()),
        }),
    ).await
    .map_err(|cause| AssignClusterError::SendingToPeerFailed {
        peer_id,
        cause: cause.to_string()
    })?;

    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use googletest::prelude::*;
    use rstest::*;

    use opendut_types::peer::{PeerLocation, PeerName, PeerNetworkConfiguration};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::topology::{DeviceDescription, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

    use crate::resources::manager::ResourcesManager;

    use super::*;

    mod store_peer_descriptor {
        use opendut_types::topology::{DeviceDescription, DeviceName};
        use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceName};

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_update_expected_resources(fixture: Fixture) -> anyhow::Result<()> {

            let resources_manager = fixture.resources_manager;

            store_peer_descriptor(StorePeerDescriptorParams {
                resources_manager: Arc::clone(&resources_manager),
                vpn: Clone::clone(&fixture.vpn),
                peer_descriptor: Clone::clone(&fixture.peer_a_descriptor),
            }).await?;

            assert_that!(resources_manager.get::<PeerDescriptor>(fixture.peer_a_id).await.as_ref(), some(eq(&fixture.peer_a_descriptor)));
            assert_that!(resources_manager.get::<DeviceDescriptor>(fixture.peer_a_device_1).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[0])));
            assert_that!(resources_manager.get::<DeviceDescriptor>(fixture.peer_a_device_2).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[1])));

            let additional_device_id = DeviceId::random();
            let additional_device = DeviceDescriptor {
                id: additional_device_id,
                name: DeviceName::try_from("PeerA_Device_42").unwrap(),
                description: DeviceDescription::try_from("Additional device for peerA").ok(),
                interface: NetworkInterfaceDescriptor {
                    name: NetworkInterfaceName::try_from("eth1").unwrap(),
                    configuration: NetworkInterfaceConfiguration::Ethernet,
                },
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
            }).await?;

            assert_that!(resources_manager.get::<PeerDescriptor>(fixture.peer_a_id).await.as_ref(), some(eq(&changed_descriptor)));
            assert_that!(resources_manager.get(fixture.peer_a_device_1).await.as_ref(), some(eq(&fixture.peer_a_descriptor.topology.devices[0])));
            assert_that!(resources_manager.get(additional_device_id).await.as_ref(), some(eq(&additional_device)));
            assert_that!(resources_manager.get(fixture.peer_a_device_2).await.as_ref(), none());

            Ok(())
        }
    }

    mod assign_cluster {
        use std::net::IpAddr;
        use std::str::FromStr;
        use opendut_carl_api::proto::services::peer_messaging_broker::Pong;
        use opendut_types::cluster::{ClusterAssignment, ClusterId};
        use crate::peer::broker::PeerMessagingBrokerOptions;
        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_update_peer_configuration(fixture: Fixture) -> anyhow::Result<()> {

            let settings = crate::settings::load_defaults()?;
            let peer_id = fixture.peer_a_id;

            let resources_manager = fixture.resources_manager;
            let peer_messaging_broker = PeerMessagingBroker::new(
                Arc::clone(&resources_manager),
                PeerMessagingBrokerOptions::load(&settings.config).unwrap(),
            );

            let peer_configuration = PeerConfiguration {
                executors: ExecutorDescriptors { executors: vec![] },
                cluster_assignment: None,
            };
            resources_manager.resources_mut(|resources| {
                resources.insert(peer_id, Clone::clone(&peer_configuration));
            }).await;

            let (_, mut receiver) = peer_messaging_broker.open(peer_id, IpAddr::from_str("1.2.3.4")?).await;
            let received = receiver.recv().await.unwrap()
                .message.unwrap();
            assert_that!(
                received,
                eq(downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                    configuration: Some(Clone::clone(&peer_configuration).into()),
                }))
            );


            let cluster_assignment = ClusterAssignment {
                id: ClusterId::random(),
                leader: PeerId::random(),
                assignments: vec![],
            };


            assign_cluster(AssignClusterParams {
                resources_manager: Arc::clone(&resources_manager),
                peer_messaging_broker: Arc::clone(&peer_messaging_broker),
                peer_id,
                cluster_assignment: Clone::clone(&cluster_assignment),
            }).await?;


            let peer_configuration = PeerConfiguration {
                cluster_assignment: Some(cluster_assignment),
                ..peer_configuration
            };

            assert_that!(
                resources_manager.get::<PeerConfiguration>(peer_id).await.as_ref(),
                some(eq(&peer_configuration))
            );


            let received = receiver.recv().await.unwrap()
                .message.unwrap();

            assert_that!(
                received,
                eq(downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                    configuration: Some(Clone::clone(&peer_configuration).into()),
                }))
            );

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
            location: PeerLocation::try_from("Ulm").ok(),
            network_configuration: PeerNetworkConfiguration {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ]
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: peer_a_device_1,
                        name: DeviceName::try_from("PeerA_Device_1").unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: NetworkInterfaceDescriptor {
                            name: NetworkInterfaceName::try_from("eth0").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                        tags: vec![],
                    },
                    DeviceDescriptor {
                        id: peer_a_device_2,
                        name: DeviceName::try_from("PeerA_Device_2").unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: NetworkInterfaceDescriptor {
                            name: NetworkInterfaceName::try_from("eth1").unwrap(),
                            configuration: NetworkInterfaceConfiguration::Ethernet,
                        },
                        tags: vec![],
                    }
                ]
            },
            executors: ExecutorDescriptors {
                executors: vec![],
            }
        };
        Fixture {
            resources_manager: ResourcesManager::new(),
            vpn: Vpn::Disabled,
            peer_a_id,
            peer_a_descriptor,
            peer_a_device_1,
            peer_a_device_2,
        }
    }
}
