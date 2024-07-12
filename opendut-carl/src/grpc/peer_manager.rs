use std::sync::Arc;
use pem::Pem;

use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;
use tracing::trace;
use url::Url;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::UserId;
use opendut_carl_api::carl::peer::{GetPeerDescriptorError, GetPeerStateError};
use opendut_carl_api::proto::services::peer_manager;
use opendut_carl_api::proto::services::peer_manager::*;
use opendut_carl_api::proto::services::peer_manager::peer_manager_server::{PeerManager as PeerManagerService, PeerManagerServer};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::cleo::{CleoId};

use crate::actions;
use crate::actions::{DeletePeerDescriptorParams, GenerateCleoSetupParams, GeneratePeerSetupParams, GetPeerStateParams, ListDevicesParams, ListPeerDescriptorsParams, StorePeerDescriptorParams};
use crate::grpc::extract;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

pub struct PeerManagerFacade {
    resources_manager: ResourcesManagerRef,
    vpn: Vpn,
    carl_url: Url,
    ca: Pem,
    oidc_registration_client: Option<RegistrationClientRef>,
}

impl PeerManagerFacade {

    pub fn new(
        resources_manager: ResourcesManagerRef,
        vpn: Vpn,
        carl_url: Url,
        ca: Pem,
        oidc_registration_client: Option<RegistrationClientRef>,
    ) -> Self {
        PeerManagerFacade {
            resources_manager,
            vpn,
            carl_url,
            ca,
            oidc_registration_client,
        }
    }

    pub fn into_grpc_service(self) -> CorsGrpcWeb<PeerManagerServer<Self>> {
        tonic_web::enable(PeerManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl PeerManagerService for PeerManagerFacade {

    #[tracing::instrument(skip_all, level="trace")]
    async fn store_peer_descriptor(&self, request: Request<StorePeerDescriptorRequest>) -> Result<Response<StorePeerDescriptorResponse>, Status> {

        let request = request.into_inner();
        let peer_descriptor: PeerDescriptor = extract!(request.peer)?;

        trace!("Received request to store peer descriptor: {peer_descriptor:?}");

        let result = actions::store_peer_descriptor(StorePeerDescriptorParams {
            resources_manager: Arc::clone(&self.resources_manager),
            vpn: Clone::clone(&self.vpn),
            peer_descriptor: Clone::clone(&peer_descriptor),
        }).await;

        match result {
            Err(error) => {
                Ok(Response::new(StorePeerDescriptorResponse {
                    reply: Some(store_peer_descriptor_response::Reply::Failure(error.into()))
                }))
            }
            Ok(peer_id) => {
                Ok(Response::new(StorePeerDescriptorResponse {
                    reply: Some(store_peer_descriptor_response::Reply::Success(
                        StorePeerDescriptorSuccess {
                            peer_id: Some(peer_id.into())
                        }
                    ))
                }))
            }
        }
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_peer_descriptor(&self, request: Request<DeletePeerDescriptorRequest>) -> Result<Response<DeletePeerDescriptorResponse>, Status> {

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        trace!("Received request to delete peer descriptor for peer <{peer_id}>.");

        let result =
            actions::delete_peer_descriptor(DeletePeerDescriptorParams {
                resources_manager: Arc::clone(&self.resources_manager),
                vpn: Clone::clone(&self.vpn),
                peer: peer_id,
                oidc_registration_client: self.oidc_registration_client.clone(),
            }).await;

        match result {
            Err(error) => {
                Ok(Response::new(DeletePeerDescriptorResponse {
                    reply: Some(delete_peer_descriptor_response::Reply::Failure(error.into()))
                }))
            }
            Ok(peer) => {
                Ok(Response::new(DeletePeerDescriptorResponse {
                    reply: Some(peer_manager::delete_peer_descriptor_response::Reply::Success(
                        DeletePeerDescriptorSuccess {
                            peer_id: Some(peer.id.into())
                        }
                    ))
                }))
            }
        }
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_peer_descriptor(&self, request: Request<GetPeerDescriptorRequest>) -> Result<Response<GetPeerDescriptorResponse>, Status> {

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        trace!("Received request to get peer descriptor for peer <{peer_id}>.");

        let result =
            actions::list_peer_descriptors(ListPeerDescriptorsParams {
                resources_manager: Arc::clone(&self.resources_manager),
            }).await
            .map_err(|error| GetPeerDescriptorError::Internal { peer_id, cause: error.to_string() })
            .and_then(|peers| peers.iter()
                .find(|peer| peer.id == peer_id)
                .ok_or_else(|| GetPeerDescriptorError::PeerNotFound { peer_id })
                .cloned()
            );

        match result {
            Err(error) => {
                Ok(Response::new(GetPeerDescriptorResponse {
                    reply: Some(get_peer_descriptor_response::Reply::Failure(error.into()))
                }))
            }
            Ok(descriptor) => {
                Ok(Response::new(GetPeerDescriptorResponse {
                    reply: Some(get_peer_descriptor_response::Reply::Success(
                        GetPeerDescriptorSuccess {
                            descriptor: Some(descriptor.into())
                        }
                    ))
                }))
            }
        }
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_peer_descriptors(&self, _: Request<ListPeerDescriptorsRequest>) -> Result<Response<ListPeerDescriptorsResponse>, Status> {

        trace!("Received request to list peer descriptors.");

        let result =
            actions::list_peer_descriptors(ListPeerDescriptorsParams {
                resources_manager: Arc::clone(&self.resources_manager),
            }).await
            .map(|peers| peers.into_iter()
                .map(From::from)
                .collect::<Vec<_>>()
            );

        match result {
            Err(error) => {
                Ok(Response::new(ListPeerDescriptorsResponse {
                    reply: Some(list_peer_descriptors_response::Reply::Failure(error.into()))
                }))
            }
            Ok(peers) => {
                Ok(Response::new(ListPeerDescriptorsResponse {
                    reply: Some(list_peer_descriptors_response::Reply::Success(
                        ListPeerDescriptorsSuccess {
                            peers
                        }
                    ))
                }))
            }
        }
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_peer_state(&self, request: Request<GetPeerStateRequest>) -> Result<Response<GetPeerStateResponse>, Status> {

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        trace!("Received request to get peer state for peer <{peer_id}>.");

        let result =
            actions::get_peer_state(GetPeerStateParams {
                peer: peer_id,
                resources_manager: Arc::clone(&self.resources_manager),
            }).await
                .map_err(|error| GetPeerStateError::Internal { peer_id, cause: error.to_string() });

        match result {
            Err(error) => {
                Ok(Response::new(GetPeerStateResponse {
                    reply: Some(get_peer_state_response::Reply::Failure(error.into()))
                }))
            }
            Ok(state) => {
                Ok(Response::new(GetPeerStateResponse {
                    reply: Some(get_peer_state_response::Reply::Success(
                        GetPeerStateSuccess {
                            state: Some(state.into())
                        }
                    ))
                }))
            }
        }
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_devices(&self, _: Request<ListDevicesRequest>) -> Result<Response<ListDevicesResponse>, Status> {

        trace!("Received request to list devices.");

        let devices = actions::list_devices(ListDevicesParams {
            resources_manager: Arc::clone(&self.resources_manager),
        }).await.expect("Devices should be listable");

        let devices = devices.into_iter()
            .map(From::from)
            .collect();

        Ok(Response::new(ListDevicesResponse { devices }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn generate_peer_setup(&self, request: Request<GeneratePeerSetupRequest>) -> Result<Response<GeneratePeerSetupResponse>, Status> { // TODO: Refactor error types.
        trace!("Received request to generate peer setup.");

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer)?;
        let user_id = UserId { value: request.user_id };

        let setup = actions::generate_peer_setup(GeneratePeerSetupParams {
            resources_manager: Arc::clone(&self.resources_manager),
            peer: peer_id,
            carl_url: Clone::clone(&self.carl_url),
            ca: Clone::clone(&self.ca),
            vpn: Clone::clone(&self.vpn),
            oidc_registration_client: self.oidc_registration_client.clone(),
            user_id,
        }).await.map_err(|cause| Status::internal(format!("Peer setup could not be created: {}", cause)))?;

        let response = peer_manager::generate_peer_setup_response::Reply::Success(peer_manager::GeneratePeerSetupSuccess {
            peer: Some(peer_id.into()),
            setup: Some(setup.into()),
        });

        Ok(Response::new(GeneratePeerSetupResponse { reply: Some(response) }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn generate_cleo_setup(&self, request: Request<GenerateCleoSetupRequest>) -> Result<Response<GenerateCleoSetupResponse>, Status> {
        trace!("Received request to generate CLEO Setup information.");

        let request = request.into_inner();
        
        let cleo_id = CleoId::random();
        let setup = actions::generate_cleo_setup(GenerateCleoSetupParams {
            cleo: cleo_id,
            carl_url: Clone::clone(&self.carl_url),
            ca: Clone::clone(&self.ca),
            oidc_registration_client: self.oidc_registration_client.clone(),
            user_id: UserId { value: request.user_id },
        }).await.map_err(|cause| Status::internal(format!("Cleo setup could not be created: {}", cause)))?;
        
        let response = generate_cleo_setup_response::Reply::Success(GenerateCleoSetupSuccess { 
            cleo: Some(cleo_id.into()), 
            setup: Some(setup.into()) 
        });

        Ok(Response::new(GenerateCleoSetupResponse { reply: Some(response) }))
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Error while loading PeerManagerFacadeOptions: {message}")]
pub struct PeerManagerFacadeOptionsLoadError {
    message: String,
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use googletest::prelude::*;
    use rstest::rstest;
    use url::Url;

    use opendut_types::peer::{PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorKind, ExecutorDescriptors, ExecutorDescriptor, ExecutorId};
    use opendut_types::proto;
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};
    use opendut_auth_tests::registration_client;

    use crate::resources::manager::ResourcesManager;
    use crate::vpn::Vpn;

    use super::*;

    pub fn get_cert() -> Pem {
        match Pem::from_str(CERTIFICATE_AUTHORITY_STRING) {
            Ok(cert) => { cert }
            Err(_) => { panic!("Not a valid certificate!") }
        }
    }

    const CERTIFICATE_AUTHORITY_STRING: &str = include_str!("../../../resources/development/tls/insecure-development-ca.pem");

    #[rstest]
    #[tokio::test]
    async fn test_successful_create_delete(#[future] registration_client: RegistrationClientRef) -> Result<()> {

        let resources_manager = ResourcesManager::new_in_memory();
        let testee = PeerManagerFacade::new(
            Arc::clone(&resources_manager),
            Vpn::Disabled,
            Url::parse("https://example.com:1234").unwrap(),
            get_cert(),
            Some(registration_client.await),
        );

        let peer_id = PeerId::random();
        let peer_descriptor = PeerDescriptor {
            id: peer_id,
            name: PeerName::try_from("TestPeer").unwrap(),
            location: PeerLocation::try_from("SiFi").ok(),
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
            topology: Topology::default(),
            executors: ExecutorDescriptors {
                executors: vec![
                    ExecutorDescriptor {
                        id: ExecutorId::random(),
                        kind: ExecutorKind::Container {
                            engine: Engine::Docker,
                            name: ContainerName::Empty,
                            image: ContainerImage::try_from("testUrl").unwrap(),
                            volumes: vec![],
                            devices: vec![],
                            envs: vec![],
                            ports: vec![],
                            command: ContainerCommand::Default,
                            args: vec![],
                        },
                        results_url: None,
                    }
                ],
            },
        };

        let create_peer_reply = testee.store_peer_descriptor(Request::new(
            StorePeerDescriptorRequest {
                peer: Some(Clone::clone(&peer_descriptor).into()),
            }
        )).await?;

        verify_that!(
            create_peer_reply.get_ref().reply,
            some(matches_pattern!(
                store_peer_descriptor_response::Reply::Success(
                    matches_pattern!(peer_manager::StorePeerDescriptorSuccess {
                        peer_id: some(eq(&proto::peer::PeerId::from(Clone::clone(&peer_id))))
                    })
                )
            ))
        )?;

        let list_reply = testee.list_peer_descriptors(Request::new(
            ListPeerDescriptorsRequest {}
        )).await?;

        verify_that!(
            list_reply.get_ref().reply,
            some(matches_pattern!(list_peer_descriptors_response::Reply::Success(
                matches_pattern!(ListPeerDescriptorsSuccess {
                    peers: container_eq([peer_descriptor.into()])
                })
            )))
        )?;

        let _ = testee.delete_peer_descriptor(Request::new(
            peer_manager::DeletePeerDescriptorRequest {
                peer_id: Some(peer_id.into()),
            }
        )).await?;

        let list_reply = testee.list_peer_descriptors(Request::new(
            peer_manager::ListPeerDescriptorsRequest {}
        )).await?;

        verify_that!(list_reply.get_ref().reply,
            some(matches_pattern!(list_peer_descriptors_response::Reply::Success(
                matches_pattern!(ListPeerDescriptorsSuccess {
                    peers: empty()
                })
            )))
        )?;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn register_fails_when_no_id_specified(#[future] registration_client: RegistrationClientRef) -> Result<()> {

        let resources_manager = ResourcesManager::new_in_memory();
        let testee = PeerManagerFacade::new(
            Arc::clone(&resources_manager),
            Vpn::Disabled,
            Url::parse("https://example.com:1234").unwrap(),
            get_cert(),
            Some(registration_client.await),
        );

        let create_peer_reply = testee.store_peer_descriptor(Request::new(
            StorePeerDescriptorRequest {
                peer: None
            }
        )).await;

        verify_that!(
            create_peer_reply,
            err(anything())
        )?;

        let list_reply = testee.list_peer_descriptors(Request::new(
            peer_manager::ListPeerDescriptorsRequest {}
        )).await?;

        verify_that!(
            list_reply.get_ref().reply,
            some(matches_pattern!(list_peer_descriptors_response::Reply::Success(
                matches_pattern!(ListPeerDescriptorsSuccess {
                    peers: empty()
                })
            )))
        )?;

        Ok(())
    }

    #[rstest]
    #[tokio::test]
    async fn unregister_fails_when_no_id_specified(#[future] registration_client: RegistrationClientRef) -> Result<()> {

        let resources_manager = ResourcesManager::new_in_memory();
        let testee = PeerManagerFacade::new(
            Arc::clone(&resources_manager),
            Vpn::Disabled,
            Url::parse("https://example.com:1234").unwrap(),
            get_cert(),
            Some(registration_client.await),
        );

        let delete_peer_reply = testee.delete_peer_descriptor(Request::new(
            peer_manager::DeletePeerDescriptorRequest {
                peer_id: None,
            }
        )).await;

        verify_that!(
            delete_peer_reply,
            err(anything())
        )?;

        let list_reply = testee.list_peer_descriptors(Request::new(
            peer_manager::ListPeerDescriptorsRequest {}
        )).await?;

        verify_that!(
            list_reply.get_ref().reply,
            some(matches_pattern!(list_peer_descriptors_response::Reply::Success(
                matches_pattern!(ListPeerDescriptorsSuccess {
                    peers: empty()
                })
            )))
        )?;

        Ok(())
    }
}
