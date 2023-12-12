#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;

use opendut_types::peer::{PeerId, PeerName};
use opendut_types::peer::state::{PeerState, PeerStates};
use opendut_types::topology::DeviceId;

#[derive(thiserror::Error, Debug)]
pub enum CreatePeerError {
    #[error("Peer '{actual_name}' <{actual_id}> could not be created, because peer '{other_name}' <{other_id}> is already registered with the same PeerId!")]
    PeerAlreadyExists {
        actual_id: PeerId,
        actual_name: PeerName,
        other_id: PeerId,
        other_name: PeerName
    },
    #[error("Peer '{peer_name}' <{peer_id}> could not be created, due to illegal devices:\n  {error}")]
    IllegalDevices {
        peer_id: PeerId,
        peer_name: PeerName,
        error: RegisterDevicesError
    },
    #[error("Peer '{peer_name}' <{peer_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeletePeerError {
    #[error("Peer <{peer_id}> could not be deleted, because a peer with that id does not exist!")]
    PeerNotFound {
        peer_id: PeerId
    },
    #[error("Peer '{peer_name}' <{peer_id}> cannot be deleted in state '{actual_state}'! A peer can be deleted when: {required_states}")]
    IllegalPeerState {
        peer_id: PeerId,
        peer_name: PeerName,
        actual_state: PeerState,
        required_states: PeerStates,
    },
    #[error("Peer '{peer_name}' <{peer_id}> could not be deleted, due to illegal devices:\n  {error}")]
    IllegalDevices {
        peer_id: PeerId,
        peer_name: PeerName,
        error: UnregisterDevicesError
    },
    #[error("Peer '{peer_name}' <{peer_id}> deleted with internal errors:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPeerError {
    #[error("A peer with id <{peer_id}> could not be found!")]
    PeerNotFound {
        peer_id: PeerId
    },
    #[error("An internal error occurred searching for a peer with id <{peer_id}>:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListPeersError {
    #[error("An internal error occurred computing the list of peers:\n  {cause}")]
    Internal {
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListDevicesError {
    #[error("An internal error occurred computing the list of devices:\n  {cause}")]
    Internal {
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
#[error("{message}")]
pub struct  CreateSetupError {
    message: String,
}

#[derive(thiserror::Error, Debug)]
pub enum RegisterDevicesError {
    #[error("Device <{device_id}> already registered!")]
    DeviceAlreadyExists {
        device_id: DeviceId,
    }
}

#[derive(thiserror::Error, Debug)]
pub enum UnregisterDevicesError {
    #[error("Device <{device_id}> does not exist!")]
    DeviceNotFound {
        device_id: DeviceId,
    }
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use tonic::codegen::{Body, Bytes, StdError};

    use opendut_types::peer::{PeerDescriptor, PeerId, PeerSetup};
    use opendut_types::topology::Device;

    use crate::carl::{ClientError, extract};
    use crate::carl::peer::{CreatePeerError, CreateSetupError, GetPeerError, ListPeersError, DeletePeerError, ListDevicesError};
    use crate::proto::services::peer_manager;
    use crate::proto::services::peer_manager::peer_manager_client::PeerManagerClient;

    #[derive(Clone, Debug)]
    pub struct PeersRegistrar<T> {
        inner: PeerManagerClient<T>,
    }

    impl<T> PeersRegistrar<T>
    where T: tonic::client::GrpcService<tonic::body::BoxBody>,
          T::Error: Into<StdError>,
          T::ResponseBody: Body<Data=Bytes> + Send + 'static,
          <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: PeerManagerClient<T>) -> PeersRegistrar<T> {
            PeersRegistrar {
                inner
            }
        }

        pub async fn create_peer(&mut self, descriptor: PeerDescriptor) -> Result<PeerId, ClientError<CreatePeerError>> {

            let request = tonic::Request::new(peer_manager::CreatePeerRequest {
                peer: Some(descriptor.into()),
            });

            let response = self.inner.create_peer(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::create_peer_response::Reply::Failure(failure) => {
                    let error = CreatePeerError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::create_peer_response::Reply::Success(success) => {
                    let peer_id = extract!(success.peer_id)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn delete_peer(&mut self, peer_id: PeerId) -> Result<PeerId, ClientError<DeletePeerError>> {

            let request = tonic::Request::new(peer_manager::DeletePeerRequest {
                peer_id: Some(peer_id.into()),
            });

            let response = self.inner.delete_peer(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::delete_peer_response::Reply::Failure(failure) => {
                    let error = DeletePeerError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::delete_peer_response::Reply::Success(success) => {
                    let peer_id = extract!(success.peer_id)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn get_peer(&mut self, peer_id: PeerId) -> Result<PeerDescriptor, ClientError<GetPeerError>> {

            let request = tonic::Request::new(peer_manager::GetPeerRequest {
                peer_id: Some(peer_id.into()),
            });

            let response = self.inner.get_peer(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::get_peer_response::Reply::Failure(failure) => {
                    let error = GetPeerError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::get_peer_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }

        pub async fn list_peers(&mut self) -> Result<Vec<PeerDescriptor>, ClientError<ListPeersError>> {

            let request = tonic::Request::new(peer_manager::ListPeersRequest {});

            let response = self.inner.list_peers(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::list_peers_response::Reply::Failure(failure) => {
                    let error = ListPeersError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::list_peers_response::Reply::Success(success) => {
                    Ok(success.peers.into_iter()
                        .map(PeerDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }

        pub async fn list_devices(&mut self) -> Result<Vec<Device>, ListDevicesError> {
            let request = tonic::Request::new(peer_manager::ListDevicesRequest {});

            match self.inner.list_devices(request).await {
                Ok(response) => {
                    response.into_inner().devices
                        .into_iter()
                        .map(Device::try_from)
                        .collect::<Result<_, _>>()
                        .map_err(|cause| ListDevicesError::Internal { cause: cause.to_string() })
                },
                Err(status) => {
                    Err(ListDevicesError::Internal { cause: format!("gRPC failure: {status}") })
                },
            }
        }

        pub async fn create_peer_setup(&mut self, peer_id: PeerId) -> Result<PeerSetup, CreateSetupError> {
            let request = tonic::Request::new(
                peer_manager::CreatePeerSetupRequest {
                    peer: Some(peer_id.into())
                }
            );

            match self.inner.create_peer_setup(request).await {
                Ok(response) => {
                    match response.into_inner().reply {
                        Some(peer_manager::create_peer_setup_response::Reply::Success(peer_manager::CreatePeerSetupSuccess { setup, .. })) => {
                            setup
                                .ok_or(CreateSetupError { message: format!("Failed to create setup-string for peer <{}>! Got no PeerSetup!", peer_id) })
                                .and_then(|setup| PeerSetup::try_from(setup)
                                    .map_err(|cause| CreateSetupError { message: cause.to_string() })
                                )
                        }
                        _ => {
                            let error = CreateSetupError { message: format!("Failed to create setup-string for peer <{}>!", peer_id) };
                            log::error!("{}", error);
                            Err(error)
                        }
                    }
                }
                Err(status) => {
                    Err(CreateSetupError { message: format!("gRPC failure: {status}") })
                }
            }
        }
    }
}
