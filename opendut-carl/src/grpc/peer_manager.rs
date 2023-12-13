use std::sync::Arc;

use tonic::{Request, Response, Status};
use tonic_web::CorsGrpcWeb;
use url::Url;

use opendut_carl_api::carl::peer::GetPeerDescriptorError;
use opendut_carl_api::proto::services::peer_manager;
use opendut_carl_api::proto::services::peer_manager::*;
use opendut_carl_api::proto::services::peer_manager::peer_manager_server::{PeerManager as PeerManagerService, PeerManagerServer};
use opendut_types::peer::{PeerDescriptor, PeerId};

use crate::actions;
use crate::actions::{StorePeerDescriptorParams, CreatePeerSetupParams, DeletePeerDescriptorParams, ListDevicesParams, ListPeerDescriptorsParams};
use crate::grpc::extract;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;

pub struct PeerManagerFacade {
    resources_manager: ResourcesManagerRef,
    vpn: Vpn,
    carl_url: Url,
}

impl PeerManagerFacade {

    pub fn new(resources_manager: ResourcesManagerRef, vpn: Vpn, carl_url: Url) -> Self {
        PeerManagerFacade {
            resources_manager,
            vpn,
            carl_url,
        }
    }

    pub fn into_grpc_service(self) -> CorsGrpcWeb<PeerManagerServer<Self>> {
        tonic_web::enable(PeerManagerServer::new(self))
    }
}

#[tonic::async_trait]
impl PeerManagerService for PeerManagerFacade {

    async fn store_peer_descriptor(&self, request: Request<StorePeerDescriptorRequest>) -> Result<Response<StorePeerDescriptorResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let request = request.into_inner();
        let peer_descriptor: PeerDescriptor = extract!(request.peer)?;

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

    async fn delete_peer_descriptor(&self, request: Request<DeletePeerDescriptorRequest>) -> Result<Response<DeletePeerDescriptorResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

        let result =
            actions::delete_peer_descriptor(DeletePeerDescriptorParams {
                resources_manager: Arc::clone(&self.resources_manager),
                vpn: Clone::clone(&self.vpn),
                peer: peer_id,
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

    async fn get_peer_descriptor(&self, request: Request<GetPeerDescriptorRequest>) -> Result<Response<GetPeerDescriptorResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let request = request.into_inner();
        let peer_id: PeerId = extract!(request.peer_id)?;

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

    async fn list_peer_descriptors(&self, request: Request<ListPeerDescriptorsRequest>) -> Result<Response<ListPeerDescriptorsResponse>, Status> {

        log::trace!("Received request: {:?}", request);

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

    async fn create_peer_setup(&self, request: Request<CreatePeerSetupRequest>) -> Result<Response<CreatePeerSetupResponse>, Status> { // TODO: Refactor error types.

        log::trace!("Received request: {:?}", request);

        let message = request.into_inner();
        let response = match message.peer {
            Some(peer_id) => {
                let peer_id = PeerId::try_from(peer_id)
                    .map_err(|cause| Status::invalid_argument(format!("PeerId could not be converted: {}", cause)))?;
                let setup = actions::create_peer_setup(CreatePeerSetupParams {
                    resources_manager: Arc::clone(&self.resources_manager),
                    peer: peer_id,
                    carl_url: Clone::clone(&self.carl_url),
                    vpn: Clone::clone(&self.vpn),
                }).await.map_err(|cause| Status::internal(format!("Peer setup could not be created: {}", cause)))?;

                peer_manager::create_peer_setup_response::Reply::Success(peer_manager::CreatePeerSetupSuccess { peer: Some(peer_id.into()), setup: Some(setup.into()) })
            }
            None => {
                peer_manager::create_peer_setup_response::Reply::Failure(peer_manager::CreatePeerSetupFailure {})
            }
        };

        Ok(Response::new(CreatePeerSetupResponse { reply: Some(response) }))
    }

    async fn list_devices(&self, request: Request<ListDevicesRequest>) -> Result<Response<ListDevicesResponse>, Status> {

        log::trace!("Received request: {:?}", request);

        let devices = actions::list_devices(ListDevicesParams {
            resources_manager: Arc::clone(&self.resources_manager),
        }).await.expect("Devices should be listable");

        let devices = devices.into_iter()
            .map(From::from)
            .collect();

        Ok(Response::new(ListDevicesResponse { devices }))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use googletest::prelude::*;
    use url::Url;

    use opendut_types::peer::PeerName;
    use opendut_types::proto;
    use opendut_types::topology::Topology;

    use crate::resources::manager::ResourcesManager;
    use crate::vpn::Vpn;

    use super::*;

    #[tokio::test]
    async fn test_successful_create_delete() -> Result<()> {

        let resources_manager = Arc::new(ResourcesManager::new());
        let testee = PeerManagerFacade::new(Arc::clone(&resources_manager), Vpn::Disabled, Url::parse("https://example.com:1234").unwrap());

        let peer_id = PeerId::random();
        let peer_descriptor = PeerDescriptor {
            id: peer_id.into(),
            name: PeerName::try_from("TestPeer").unwrap(),
            topology: Topology::default(),
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
                        peer_id: some(eq(proto::peer::PeerId::from(Clone::clone(&peer_id))))
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

    #[tokio::test]
    async fn register_fails_when_no_id_specified() -> Result<()> {

        let resources_manager = Arc::new(ResourcesManager::new());
        let testee = PeerManagerFacade::new(Arc::clone(&resources_manager), Vpn::Disabled, Url::parse("https://example.com:1234").unwrap());

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

    #[tokio::test]
    async fn unregister_fails_when_no_id_specified() -> Result<()> {

        let resources_manager = Arc::new(ResourcesManager::new());
        let testee = PeerManagerFacade::new(Arc::clone(&resources_manager), Vpn::Disabled, Url::parse("https://example.com:1234").unwrap());

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
