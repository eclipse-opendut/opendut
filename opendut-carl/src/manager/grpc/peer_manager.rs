use pem::Pem;

use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::UserId;
use opendut_carl_api::carl::peer::{GetPeerDescriptorError, ListPeerDescriptorsError, ListPeerStatesError};
use opendut_carl_api::proto;
use opendut_carl_api::proto::services;
use opendut_carl_api::proto::services::peer_manager::peer_manager_server::{PeerManager as PeerManagerService, PeerManagerServer};
use opendut_carl_api::proto::services::peer_manager::*;
use opendut_types::cleo::CleoId;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;
use tracing::{error, trace};
use url::Url;

use crate::manager::grpc::extract;
use crate::manager::peer_manager;
use crate::manager::peer_manager::{DeletePeerDescriptorError, DeletePeerDescriptorParams, GenerateCleoSetupParams, GeneratePeerSetupParams, StorePeerDescriptorError, StorePeerDescriptorParams};
use crate::resource::manager::ResourceManagerRef;
use crate::resource::persistence::error::{MapToInner, PersistenceError};
use crate::settings::vpn::Vpn;

pub struct PeerManagerFacade {
    resource_manager: ResourceManagerRef,
    vpn: Vpn,
    carl_url: Url,
    ca: Pem,
    oidc_registration_client: Option<RegistrationClientRef>,
}

impl PeerManagerFacade {

    pub fn new(
        resource_manager: ResourceManagerRef,
        vpn: Vpn,
        carl_url: Url,
        ca: Pem,
        oidc_registration_client: Option<RegistrationClientRef>,
    ) -> Self {
        PeerManagerFacade {
            resource_manager,
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
        let peer: PeerDescriptor = extract!(request.peer)?;

        trace!("Received request to store peer descriptor: {peer:?}");

        let result =
            self.resource_manager.resources_mut(async |resources| {
                resources.store_peer_descriptor(StorePeerDescriptorParams {
                    vpn: Clone::clone(&self.vpn),
                    peer_descriptor: Clone::clone(&peer),
                }).await
            }).await
            .map_to_inner(|source| StorePeerDescriptorError::Persistence {
                peer_id: peer.id,
                peer_name: peer.name,
                source: source.context("Persistence error in transaction for storing peer descriptor"),
            })
            .inspect_err(|error| error!("{error}"))
            .map_err(opendut_carl_api::carl::peer::StorePeerDescriptorError::from);

        let reply = match result {
            Ok(peer_id) => store_peer_descriptor_response::Reply::Success(
                StorePeerDescriptorSuccess {
                    peer_id: Some(peer_id.into())
                }
            ),
            Err(error) => store_peer_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(StorePeerDescriptorResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn delete_peer_descriptor(&self, request: Request<DeletePeerDescriptorRequest>) -> Result<Response<DeletePeerDescriptorResponse>, Status> {

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        trace!("Received request to delete peer descriptor for peer <{peer_id}>.");

        let result =
            self.resource_manager.resources_mut(async |resources|
                resources.delete_peer_descriptor(DeletePeerDescriptorParams {
                    vpn: Clone::clone(&self.vpn),
                    peer: peer_id,
                    oidc_registration_client: self.oidc_registration_client.clone(),
                }).await
            ).await
            .map_to_inner(|source| DeletePeerDescriptorError::Persistence {
                peer_id,
                peer_name: None,
                source: source.context("Persistence error in transaction for deleting peer descriptor"),
            })
            .inspect_err(|error| error!("{error}"))
            .map_err(opendut_carl_api::carl::peer::DeletePeerDescriptorError::from);

        let response = match result {
            Ok(peer) => proto::services::peer_manager::delete_peer_descriptor_response::Reply::Success(
                DeletePeerDescriptorSuccess {
                    peer_id: Some(peer.id.into())
                }
            ),
            Err(error) => delete_peer_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(DeletePeerDescriptorResponse {
            reply: Some(response),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_peer_descriptor(&self, request: Request<GetPeerDescriptorRequest>) -> Result<Response<GetPeerDescriptorResponse>, Status> {

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        trace!("Received request to get peer descriptor for peer <{peer_id}>.");

        let result =
            self.resource_manager.get::<PeerDescriptor>(peer_id).await
                .inspect_err(|error| error!("Error while getting peer descriptor from gRPC API: {error}"))
                .map_err(|_: PersistenceError| GetPeerDescriptorError::Internal { peer_id, cause: String::from("Error when accessing persistence while getting peer descriptor") });

        let response = match result {
            Ok(descriptor) => match descriptor {
                Some(descriptor) => get_peer_descriptor_response::Reply::Success(
                    GetPeerDescriptorSuccess {
                        descriptor: Some(descriptor.into())
                    }
                ),
                None => get_peer_descriptor_response::Reply::Failure(
                    GetPeerDescriptorError::PeerNotFound { peer_id }.into()
                ),
            }
            Err(error) => get_peer_descriptor_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(GetPeerDescriptorResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_peer_descriptors(&self, _: Request<ListPeerDescriptorsRequest>) -> Result<Response<ListPeerDescriptorsResponse>, Status> {

        trace!("Received request to list peer descriptors.");

        let result = self.resource_manager.list::<PeerDescriptor>().await
            .inspect_err(|error| error!("Error while listing peer descriptors from gRPC API: {error}"))
            .map_err(|_: PersistenceError| ListPeerDescriptorsError::Internal { cause: String::from("Error when accessing persistence while listing peer descriptors") });

        let response = match result {
            Ok(peers) => {
                let peers = peers.into_values()
                    .map(From::from)
                    .collect::<Vec<_>>();

                list_peer_descriptors_response::Reply::Success(
                    ListPeerDescriptorsSuccess { peers }
                )
            }
            Err(error) => list_peer_descriptors_response::Reply::Failure(error.into())
        };

        Ok(Response::new(ListPeerDescriptorsResponse {
            reply: Some(response)
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn get_peer_state(&self, request: Request<GetPeerStateRequest>) -> Result<Response<GetPeerStateResponse>, Status> {

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        trace!("Received request to get peer state for peer <{peer_id}>.");

        let result =
            self.resource_manager.resources(async |resources| {
                resources.get_peer_state(peer_id)
            }).await
            .inspect_err(|error| error!("{error}"))
            .map_err(opendut_carl_api::carl::peer::GetPeerStateError::from);

        let reply = match result {
            Ok(state) => get_peer_state_response::Reply::Success(
                GetPeerStateSuccess {
                    state: Some(state.into())
                }
            ),
            Err(error) => get_peer_state_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(GetPeerStateResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_peer_states(&self, _: Request<ListPeerStatesRequest>) -> Result<Response<ListPeerStatesResponse>, Status> {

        trace!("Received request to list peer states.");

        let result =
            self.resource_manager.resources(async |resources|
                resources.list_peer_states()
            ).await
            .inspect_err(|error| error!("{error}"))
            .map_err(ListPeerStatesError::from);

        let reply = match result {
            Ok(peer_states) => list_peer_states_response::Reply::Success(
                ListPeerStatesSuccess {
                    peer_state_entries: peer_states.into_iter()
                        .map(|(peer_id, peer_state)| ListPeerStatesEntry {
                            peer_id: Some(peer_id.into()),
                            peer_state: Some(peer_state.into()),
                        })
                        .collect(),
                }
            ),
            Err(error) => list_peer_states_response::Reply::Failure(error.into()),
        };

        Ok(Response::new(ListPeerStatesResponse {
            reply: Some(reply),
        }))
    }

    #[tracing::instrument(skip_all, level="trace")]
    async fn list_devices(&self, _: Request<ListDevicesRequest>) -> Result<Response<ListDevicesResponse>, Status> {

        trace!("Received request to list devices.");

        let devices = self.resource_manager.resources(async |resources|
            resources.list_devices()
        ).await
            .expect("Devices should be listable");

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

        let setup = self.resource_manager.resources(async |resources|
            resources.generate_peer_setup(GeneratePeerSetupParams {
                peer: peer_id,
                carl_url: Clone::clone(&self.carl_url),
                ca: Clone::clone(&self.ca),
                vpn: Clone::clone(&self.vpn),
                oidc_registration_client: self.oidc_registration_client.clone(),
                user_id,
            }).await
        ).await
            .map_err(|cause| Status::internal(format!("Peer setup could not be created: {}", cause)))?;

        let response = services::peer_manager::generate_peer_setup_response::Reply::Success(services::peer_manager::GeneratePeerSetupSuccess {
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
        let setup = peer_manager::generate_cleo_setup(GenerateCleoSetupParams {
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

    use crate::resource::manager::ResourceManager;
    use crate::settings::vpn::Vpn;
    use opendut_auth_tests::registration_client;
    use opendut_carl_api::proto::services;
    use opendut_types::peer::executor::{container::{ContainerCommand, ContainerImage, ContainerName, Engine}, ExecutorDescriptor, ExecutorDescriptors, ExecutorId, ExecutorKind};
    use opendut_types::peer::{PeerLocation, PeerName, PeerNetworkDescriptor};
    use opendut_types::proto;
    use opendut_types::topology::Topology;
    use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    use super::*;

    pub fn get_cert() -> Pem {
        match Pem::from_str(CERTIFICATE_AUTHORITY_STRING) {
            Ok(cert) => { cert }
            Err(_) => { panic!("Not a valid certificate!") }
        }
    }

    const CERTIFICATE_AUTHORITY_STRING: &str = include_str!("../../../../resources/development/tls/insecure-development-ca.pem");

    #[rstest]
    #[tokio::test]
    async fn test_successful_create_delete(#[future] registration_client: RegistrationClientRef) -> Result<()> {

        let resource_manager = ResourceManager::new_in_memory();
        let testee = PeerManagerFacade::new(
            Arc::clone(&resource_manager),
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
                    matches_pattern!(services::peer_manager::StorePeerDescriptorSuccess {
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
            services::peer_manager::DeletePeerDescriptorRequest {
                peer_id: Some(peer_id.into()),
            }
        )).await?;

        let list_reply = testee.list_peer_descriptors(Request::new(
            services::peer_manager::ListPeerDescriptorsRequest {}
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

        let resource_manager = ResourceManager::new_in_memory();
        let testee = PeerManagerFacade::new(
            Arc::clone(&resource_manager),
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
            services::peer_manager::ListPeerDescriptorsRequest {}
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

        let resource_manager = ResourceManager::new_in_memory();
        let testee = PeerManagerFacade::new(
            Arc::clone(&resource_manager),
            Vpn::Disabled,
            Url::parse("https://example.com:1234").unwrap(),
            get_cert(),
            Some(registration_client.await),
        );

        let delete_peer_reply = testee.delete_peer_descriptor(Request::new(
            services::peer_manager::DeletePeerDescriptorRequest {
                peer_id: None,
            }
        )).await;

        verify_that!(
            delete_peer_reply,
            err(anything())
        )?;

        let list_reply = testee.list_peer_descriptors(Request::new(
            services::peer_manager::ListPeerDescriptorsRequest {}
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
