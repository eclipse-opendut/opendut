use std::fmt::{Display, Formatter};
use std::ops::Not;
use std::sync::Arc;

use pem::Pem;
use tracing::{debug, error, info, Span, warn};
use url::Url;
use uuid::Uuid;

use opendut_auth::registration::client::RegistrationClientRef;
pub use opendut_carl_api::carl::peer::{
    DeletePeerDescriptorError,
    IllegalDevicesError,
    ListDevicesError,
    ListPeerDescriptorsError,
    StorePeerDescriptorError,
    GetPeerStateError
};
use opendut_carl_api::proto::services::peer_messaging_broker::{ApplyPeerConfiguration, downstream};
use opendut_types::{peer, proto};
use opendut_types::cleo::{CleoId, CleoSetup};
use opendut_types::cluster::ClusterAssignment;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerSetup};
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2, PeerNetworkConfiguration};
use opendut_types::peer::state::{PeerBlockedState, PeerState, PeerUpState};
use opendut_types::proto::peer::configuration::{peer_configuration_parameter, PeerConfigurationParameterExecutor, PeerConfigurationParameterTargetPresent};
use opendut_types::topology::{DeviceDescriptor, DeviceId};
use opendut_types::util::net::{AuthConfig, Certificate, ClientCredentials, NetworkInterfaceName};
use opendut_types::vpn::VpnPeerConfiguration;
use opendut_util::ErrorOr;

use crate::peer::broker::{PeerMessagingBroker, PeerMessagingBrokerRef};
use crate::persistence::error::PersistenceError;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

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

            resources.insert(peer_id, PeerState::Down)?;
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

pub struct DeletePeerDescriptorParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer: PeerId,
    pub oidc_registration_client: Option<RegistrationClientRef>,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_peer_descriptor(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

    async fn inner(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

        let peer_id = params.peer;
        let resources_manager = params.resources_manager;

        debug!("Deleting peer descriptor of peer <{peer_id}>.");

        let peer_descriptor = resources_manager.resources_mut(|resources| {

            let peer_descriptor = resources.remove::<PeerDescriptor>(peer_id)
                .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: None, cause: cause.to_string() })?
                .ok_or_else(|| DeletePeerDescriptorError::PeerNotFound { peer_id })?;

            let peer_name = &peer_descriptor.name;

            peer_descriptor.topology.devices.iter().try_for_each(|device| {
                let device_id = device.id;
                let device_name = &device.name;
                resources.remove::<DeviceDescriptor>(device_id)
                    .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: Some(peer_name.clone()), cause: cause.to_string() })?;
                debug!("Deleted device '{device_name}' <{device_id}> of peer '{peer_name}' <{peer_id}>.");
                Ok(())
            })?;

            Ok(peer_descriptor)
        }).await?;

        let peer_name = &peer_descriptor.name;

        if let Some(registration_client) = params.oidc_registration_client {
            let resource_id = peer_id.into();
            debug!("Deleting OIDC client for peer '{peer_name}' <{peer_id}>.");
            let deleted_clients = registration_client.delete_client_by_resource_id(resource_id)
                .await
                .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: Some(peer_name.clone()), cause: cause.to_string() })?;
            let deleted_client_ids =  deleted_clients.value().into_iter().map(|client| client.client_id).collect::<Vec<String>>();
            debug!("Successfully deleted oidc clients for peer '{peer_name}' <{peer_id}>. OIDC client_ids='{}'.", deleted_client_ids.join(","));
        };
        
        if let Vpn::Enabled { vpn_client } = params.vpn {
            debug!("Deleting vpn peer <{peer_id}>.");
            vpn_client.delete_peer(peer_id)
                .await
                .map_err(|cause| DeletePeerDescriptorError::Internal {
                    peer_id,
                    peer_name: Some(peer_name.clone()),
                    cause: cause.to_string()
                })?;
            info!("Successfully deleted VPN peer <{peer_id}>.");
        } else {
            warn!("VPN disabled. Skipping VPN peer deletion!");
        }

        info!("Successfully deleted peer descriptor of '{peer_name}' <{peer_id}>.");

        Ok(peer_descriptor)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct ListPeerDescriptorsParams {
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_peer_descriptors(params: ListPeerDescriptorsParams) -> Result<Vec<PeerDescriptor>, ListPeerDescriptorsError> {

    async fn inner(params: ListPeerDescriptorsParams) -> Result<Vec<PeerDescriptor>, ListPeerDescriptorsError> {

        let resources_manager = params.resources_manager;

        debug!("Querying all peer descriptors.");

        let peers = resources_manager.resources(|resources| {
            resources.list::<PeerDescriptor>()
        }).await
        .map_err(|cause| ListPeerDescriptorsError::Internal { cause: cause.to_string() })?;

        info!("Successfully queried all peer descriptors.");

        Ok(peers)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct GetPeerStateParams {
    pub peer: PeerId,
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn get_peer_state(params: GetPeerStateParams) -> Result<PeerState, GetPeerStateError> {

    async fn inner(params: GetPeerStateParams) -> Result<PeerState, GetPeerStateError> {

        let peer_id = params.peer;
        let resources_manager = params.resources_manager;

        debug!("Querying state of peer.");
        
        let peer_state = resources_manager.resources_mut(|resources| {
            resources.get::<PeerState>(peer_id)
        }).await
            .map_err(|cause| GetPeerStateError::Internal {  peer_id ,cause: cause.to_string() })?
            .ok_or(GetPeerStateError::PeerNotFound { peer_id })?;
        
        
        info!("Successfully queried state of peer.");

        Ok(peer_state)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct ListDevicesParams {
    pub resources_manager: ResourcesManagerRef,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn list_devices(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

    async fn inner(params: ListDevicesParams) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {

        let resources_manager = params.resources_manager;

        debug!("Querying all devices.");

        let devices = resources_manager.resources(|resource| {
            resource.list::<DeviceDescriptor>()
        }).await
        .map_err(|cause| ListDevicesError::Internal { cause: cause.to_string() })?;

        info!("Successfully queried all peers.");

        Ok(devices)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct GeneratePeerSetupParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer: PeerId,
    pub carl_url: Url,
    pub ca: Pem,
    pub vpn: Vpn,
    pub oidc_registration_client: Option<RegistrationClientRef>,
}

#[derive(thiserror::Error, Debug)]
pub enum GeneratePeerSetupError {
    #[error("A PeerSetup for peer <{0}> could not be created, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("An error occurred while accessing persistence for creating a PeerSetup for peer <{peer_id}>")]
    Persistance { peer_id: PeerId, #[source] source: PersistenceError },
    #[error("An internal error occurred while creating a PeerSetup for peer '{peer_name}' <{peer_id}>:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn generate_peer_setup(params: GeneratePeerSetupParams, user_id: String) -> Result<PeerSetup, GeneratePeerSetupError> {

    async fn inner(params: GeneratePeerSetupParams, user_id: String) -> Result<PeerSetup, GeneratePeerSetupError> {

        let peer_id = params.peer;

        debug!("Generating PeerSetup for peer <{peer_id}>");

        let peer_descriptor = params.resources_manager.get::<PeerDescriptor>(peer_id).await
            .map_err(|source| GeneratePeerSetupError::Persistance { peer_id, source })?
            .ok_or(GeneratePeerSetupError::PeerNotFound(peer_id))?;

        let peer_name = peer_descriptor.name;

        let vpn_config = if let Vpn::Enabled { vpn_client } = &params.vpn {
            debug!("Retrieving VPN configuration for peer <{peer_id}>.");
            let vpn_config = vpn_client.generate_vpn_peer_configuration(params.peer).await
                .map_err(|cause| GeneratePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
            info!("Successfully retrieved vpn configuration for peer <{peer_id}>.");
            vpn_config
        }
        else {
            warn!("VPN is disabled. PeerSetup for peer '{peer_name}' <{peer_id}> will not contain any VPN information!");
            VpnPeerConfiguration::Disabled
        };

        let auth_config = match params.oidc_registration_client {
            None => {
                AuthConfig::Disabled
            }
            Some(registration_client) => {
                let resource_id = peer_id.into();
                debug!("Generating OIDC client for peer '{peer_name}' <{peer_id}>.");
                let issuer_url = registration_client.config.issuer_remote_url.clone();
                let client_credentials = registration_client.register_new_client_for_user(resource_id, user_id)
                    .await
                    .map_err(|cause| GeneratePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
                debug!("Successfully generated peer setup for peer '{peer_name}' <{peer_id}>. OIDC client_id='{}'.", client_credentials.client_id.clone().value());
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

    inner(params, user_id).await
        .inspect_err(|err| error!("{err}"))
}

pub struct GenerateCleoSetupParams {
    pub cleo: CleoId,
    pub carl_url: Url,
    pub ca: Pem,
    pub oidc_registration_client: Option<RegistrationClientRef>,
}

#[derive(thiserror::Error, Debug)]
pub enum GenerateCleoSetupError {
    #[error("An internal error occurred while creating a CleoSetup:\n  {cause}")]
    Internal {
        cause: String,
    }
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn generate_cleo_setup(params: GenerateCleoSetupParams, user_id: String) -> Result<CleoSetup, GenerateCleoSetupError> {

    async fn inner(params: GenerateCleoSetupParams, user_id: String) -> Result<CleoSetup, GenerateCleoSetupError> {

        let cleo_id = params.cleo;
        debug!("Generating CleoSetup");

        let auth_config = match params.oidc_registration_client {
            None => {
                AuthConfig::Disabled
            }
            Some(registration_client) => {
                let resource_id = cleo_id.into();
                debug!("Generating OIDC client for CLEO: <{cleo_id}>.");
                let issuer_url = registration_client.config.issuer_remote_url.clone();
                let client_credentials = registration_client.register_new_client_for_user(resource_id, user_id)
                    .await
                    .map_err(|cause| GenerateCleoSetupError::Internal { cause: cause.to_string() })?;
                debug!("Successfully generated cleo setup with id <{cleo_id}>. OIDC client_id='{}'.", client_credentials.client_id.clone().value());
                AuthConfig::from_credentials(issuer_url, client_credentials)
            }
        };

        Ok(CleoSetup {
            id: cleo_id,
            carl: params.carl_url,
            ca: Certificate(params.ca),
            auth_config,
        })
    }

    inner(params, user_id).await
        .inspect_err(|err| error!("{err}"))
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
    #[error("Error while persisting ClusterAssignment for peer <{peer_id}>.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
}

pub async fn assign_cluster(params: AssignClusterParams) -> Result<(), AssignClusterError> {

    let peer_id = params.peer_id;

    let (peer_configuration, peer_configuration2) = params.resources_manager.resources_mut(|resources| {
        let peer_configuration = resources.get::<PeerConfiguration>(peer_id)
            .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
            .ok_or(AssignClusterError::PeerNotFound(peer_id))
            .and_then(|peer_configuration| {
                let peer_configuration = PeerConfiguration {
                    cluster_assignment: Some(params.cluster_assignment),
                    ..peer_configuration
                };
                resources.insert(peer_id, Clone::clone(&peer_configuration))
                    .map_err(|source| AssignClusterError::Persistence { peer_id, source })?;
                Ok(peer_configuration)
            })?;

        let peer_configuration2 = resources.get::<PeerConfiguration2>(peer_id)
            .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
            .ok_or(AssignClusterError::PeerNotFound(peer_id))?;

        let peer_state = resources.get::<PeerState>(peer_id)
            .map_err(|source| AssignClusterError::Persistence { peer_id, source })?
            .ok_or(AssignClusterError::PeerNotFound(peer_id))?;
        
        match peer_state {
            PeerState::Down => {}
            PeerState::Up { remote_host, .. } => {
                resources.insert(peer_id, PeerState::Up {
                    inner: PeerUpState::Blocked(PeerBlockedState::Member),
                    remote_host,
                })
                    .map_err(|source| AssignClusterError::Persistence { peer_id, source })?;
            }
        }
        
        Ok((peer_configuration, peer_configuration2))
    }).await?;

    params.peer_messaging_broker.send_to_peer(
        peer_id,
        downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
            configuration: Some(peer_configuration.into()),
            configuration2: Some(peer_configuration2.into()),
        }),
    ).await
    .map_err(|cause| AssignClusterError::SendingToPeerFailed {
        peer_id,
        cause: cause.to_string()
    })?;

    Ok(())
}

pub struct UnassignClusterParams {
    pub resources_manager: ResourcesManagerRef,
    pub peer_id: PeerId,
}


#[derive(thiserror::Error, Debug)]
pub enum UnassignClusterError {
    #[error("Unassigning cluster for peer <{0}> failed, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("Error while persisting ClusterAssignment for peer <{peer_id}>.")]
    Persistence { peer_id: PeerId, #[source] source: PersistenceError },
}

pub async fn unassign_cluster(params: UnassignClusterParams) -> Result<(), UnassignClusterError> {

    let peer_id = params.peer_id;

    params.resources_manager.resources_mut(|resources| {
        let peer_state = resources.get::<PeerState>(peer_id)
            .map_err(|source| UnassignClusterError::Persistence { peer_id, source })?
            .ok_or(UnassignClusterError::PeerNotFound(peer_id))?;

        match peer_state {
            PeerState::Down => {}
            PeerState::Up { remote_host, .. } => {
                resources.insert(peer_id, PeerState::Up {
                    inner: PeerUpState::Available,
                    remote_host,
                })
                    .map_err(|source| UnassignClusterError::Persistence { peer_id, source })?;
            }
        }

        Ok(())
    }).await?;
    
    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use googletest::prelude::*;
    use rstest::*;

    use opendut_types::peer::{PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::peer::executor::ExecutorDescriptors;
    use opendut_types::topology::{DeviceDescription, DeviceName, Topology};
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use crate::resources::manager::ResourcesManager;

    use super::*;

    mod store_peer_descriptor {
        use opendut_types::topology::{DeviceDescription, DeviceName};
        use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

        use super::*;

        #[rstest]
        #[tokio::test]
        async fn should_update_expected_resources(fixture: Fixture, store_peer_descriptor_options: StorePeerDescriptorOptions) -> anyhow::Result<()> {

            let resources_manager = fixture.resources_manager;

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
                cluster_assignment: None,
                network: PeerNetworkConfiguration {
                    bridge_name: NetworkInterfaceName::try_from("br-opendut-1").unwrap()
                }
            };
            resources_manager.resources_mut(|resources| {
                resources.insert(peer_id, Clone::clone(&peer_configuration))
            }).await?;
            let peer_configuration2 = PeerConfiguration2 {
                executors: vec![],
            };
            resources_manager.resources_mut(|resources| {
                resources.insert(peer_id, Clone::clone(&peer_configuration2))
            }).await?;

            let (_, mut receiver) = peer_messaging_broker.open(peer_id, IpAddr::from_str("1.2.3.4")?).await?;
            let received = receiver.recv().await.unwrap()
                .message.unwrap();
            assert_that!(
                received,
                eq(downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                    configuration: Some(Clone::clone(&peer_configuration).into()),
                    configuration2: Some(Clone::clone(&peer_configuration2).into()),
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
                resources_manager.get::<PeerConfiguration>(peer_id).await?.as_ref(),
                some(eq(&peer_configuration))
            );


            let received = receiver.recv().await.unwrap()
                .message.unwrap();

            assert_that!(
                received,
                eq(downstream::Message::ApplyPeerConfiguration(ApplyPeerConfiguration {
                    configuration: Some(Clone::clone(&peer_configuration).into()),
                    configuration2: Some(peer_configuration2.into()),
                }))
            );

            Ok(())
        }
    }

    mod create_setup_string {
        use std::str::FromStr;

        use opendut_auth_tests::registration_client;

        use super::*;

        #[tokio::test]
        async fn should_create_setup_string_cleo() -> anyhow::Result<()> {
            let generate_cleo_setup_params = GenerateCleoSetupParams {
                cleo: CleoId::try_from("787d0b11-51f3-4cfe-8131-c7d89d53f0e9")?,
                carl_url: Url::parse("https://example.com:1234").unwrap(),
                ca: get_cert(),
                oidc_registration_client: None,
            };

            let cleo_setup = generate_cleo_setup(generate_cleo_setup_params, String::from("testUser")).await?;
            assert_that!(cleo_setup.id, eq(CleoId::try_from("787d0b11-51f3-4cfe-8131-c7d89d53f0e9")?));
            assert_that!(cleo_setup.auth_config, eq(AuthConfig::Disabled));
            assert_that!(cleo_setup.carl, eq(Url::parse("https://example.com:1234").unwrap()));

            Ok(())
        }

        pub fn get_cert() -> Pem {
            match Pem::from_str(CERTIFICATE_AUTHORITY_STRING) {
                Ok(cert) => { cert }
                Err(_) => { panic!("Not a valid certificate!") }
            }
        }

        const CERTIFICATE_AUTHORITY_STRING: &str = include_str!("../../../resources/development/tls/insecure-development-ca.pem");
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
            network: PeerNetworkDescriptor {
                interfaces: vec![
                    NetworkInterfaceDescriptor {
                        id: NetworkInterfaceId::random(),
                        name: NetworkInterfaceName::try_from("eth0").unwrap(),
                        configuration: NetworkInterfaceConfiguration::Ethernet,
                    },
                ],
                bridge_name: Some(NetworkInterfaceName::try_from("br-opendut-1").unwrap()),
            },
            topology: Topology {
                devices: vec![
                    DeviceDescriptor {
                        id: peer_a_device_1,
                        name: DeviceName::try_from("PeerA_Device_1").unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: NetworkInterfaceId::random(),
                        tags: vec![],
                    },
                    DeviceDescriptor {
                        id: peer_a_device_2,
                        name: DeviceName::try_from("PeerA_Device_2").unwrap(),
                        description: DeviceDescription::try_from("Huii").ok(),
                        interface: NetworkInterfaceId::random(),
                        tags: vec![],
                    }
                ]
            },
            executors: ExecutorDescriptors {
                executors: vec![],
            }
        };
        Fixture {
            resources_manager: ResourcesManager::new_in_memory(),
            vpn: Vpn::Disabled,
            peer_a_id,
            peer_a_descriptor,
            peer_a_device_1,
            peer_a_device_2,
        }
    }

    #[fixture]
    fn store_peer_descriptor_options() -> StorePeerDescriptorOptions {
        StorePeerDescriptorOptions {
            bridge_name_default: NetworkInterfaceName::try_from("br-opendut").unwrap(),
        }
    }
}
