use std::fmt;
use std::fmt::Formatter;
#[cfg(any(feature = "client", feature = "wasm-client"))]
pub use client::*;
use opendut_types::peer::{PeerId, PeerName};
use opendut_types::peer::state::PeerState;
use opendut_types::ShortName;
use opendut_types::topology::DeviceId;

#[derive(thiserror::Error, Debug)]
pub enum StorePeerDescriptorError {
    #[error("Peer '{peer_name}' <{peer_id}> cannot be updated in state '{}'! A peer can be updated when: {}", actual_state.short_name(), PeerState::short_names_joined(required_states))]
    IllegalPeerState {
        peer_id: PeerId,
        peer_name: PeerName,
        actual_state: PeerState,
        required_states: Vec<PeerState>,
    },
    #[error("Peer '{peer_name}' <{peer_id}> could not be stored, due to illegal devices:\n  {error}")]
    IllegalDevices {
        peer_id: PeerId,
        peer_name: PeerName,
        error: IllegalDevicesError
    },
    #[error("Peer '{peer_name}' <{peer_id}> could not be created, due to internal errors:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeletePeerDescriptorError {
    PeerNotFound {
        peer_id: PeerId
    },
    IllegalPeerState {
        peer_id: PeerId,
        peer_name: PeerName,
        actual_state: PeerState,
        required_states: Vec<PeerState>,
    },
    Internal {
        peer_id: PeerId,
        peer_name: Option<PeerName>,
        cause: String
    }
}
impl fmt::Display for DeletePeerDescriptorError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DeletePeerDescriptorError::PeerNotFound { peer_id } => writeln!(f, "Peer <{peer_id}> could not be deleted, because a peer with that id does not exist!"),
            DeletePeerDescriptorError::IllegalPeerState { peer_id, peer_name, actual_state, required_states } => {
                let actual_state = actual_state.short_name();
                let required_states = PeerState::short_names_joined(required_states);
                writeln!(f, "Peer '{peer_name}' <{peer_id}> cannot be deleted in state '{actual_state}'! A peer can be deleted when: {required_states}")
            }
            DeletePeerDescriptorError::Internal { peer_id, peer_name, cause } => {
                let peer_name = match peer_name {
                    Some(peer_name) => format!("'{peer_name}' "),
                    None => String::new(),
                };
                writeln!(f, "Peer {peer_name}<{peer_id}> deleted with internal errors:\n  {cause}")
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPeerDescriptorError {
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
pub enum ListPeerDescriptorsError {
    #[error("An internal error occurred computing the list of peers:\n  {cause}")]
    Internal {
        cause: String
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPeerStateError {
    #[error("A peer with id <{peer_id}> could not be found!")]
    PeerNotFound {
        peer_id: PeerId
    },
    #[error("An internal error occurred searching for the state of a peer with id <{peer_id}>:\n  {cause}")]
    Internal {
        peer_id: PeerId,
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
pub enum IllegalDevicesError {
    #[error("Device <{device_id}> already registered!")]
    DeviceAlreadyExists {
        device_id: DeviceId,
    }
}

#[cfg(any(feature = "client", feature = "wasm-client"))]
mod client {
    use tonic::codegen::{Body, Bytes, http, InterceptedService, StdError};
    use tracing::error;
    use opendut_types::cleo::CleoSetup;

    use opendut_types::peer::{PeerDescriptor, PeerId, PeerSetup};
    use opendut_types::peer::state::PeerState;
    use opendut_types::topology::DeviceDescriptor;

    use crate::carl::{ClientError, extract};
    use crate::carl::peer::{DeletePeerDescriptorError, GetPeerDescriptorError, GetPeerStateError, ListDevicesError, ListPeerDescriptorsError, StorePeerDescriptorError};
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

        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> PeersRegistrar<InterceptedService<T, F>>
            where
                F: tonic::service::Interceptor,
                T::ResponseBody: Default,
                T: tonic::codegen::Service<
                    http::Request<tonic::body::BoxBody>,
                    Response = http::Response<
                        <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                    >,
                >,
                <T as tonic::codegen::Service<
                    http::Request<tonic::body::BoxBody>,
                >>::Error: Into<StdError> + Send + Sync,
        {
            let inner_client = PeerManagerClient::new(InterceptedService::new(inner, interceptor));
            PeersRegistrar {
                inner: inner_client
            }
        }

        pub async fn store_peer_descriptor(&mut self, descriptor: PeerDescriptor) -> Result<PeerId, ClientError<StorePeerDescriptorError>> {

            let request = tonic::Request::new(peer_manager::StorePeerDescriptorRequest {
                peer: Some(descriptor.into()),
            });

            let response = self.inner.store_peer_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::store_peer_descriptor_response::Reply::Failure(failure) => {
                    let error = StorePeerDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::store_peer_descriptor_response::Reply::Success(success) => {
                    let peer_id = extract!(success.peer_id)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn delete_peer_descriptor(&mut self, peer_id: PeerId) -> Result<PeerId, ClientError<DeletePeerDescriptorError>> {

            let request = tonic::Request::new(peer_manager::DeletePeerDescriptorRequest {
                peer_id: Some(peer_id.into()),
            });

            let response = self.inner.delete_peer_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::delete_peer_descriptor_response::Reply::Failure(failure) => {
                    let error = DeletePeerDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::delete_peer_descriptor_response::Reply::Success(success) => {
                    let peer_id = extract!(success.peer_id)?;
                    Ok(peer_id)
                }
            }
        }

        pub async fn get_peer_descriptor(&mut self, peer_id: PeerId) -> Result<PeerDescriptor, ClientError<GetPeerDescriptorError>> {

            let request = tonic::Request::new(peer_manager::GetPeerDescriptorRequest {
                peer_id: Some(peer_id.into()),
            });

            let response = self.inner.get_peer_descriptor(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::get_peer_descriptor_response::Reply::Failure(failure) => {
                    let error = GetPeerDescriptorError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::get_peer_descriptor_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.descriptor)?;
                    Ok(peer_descriptor)
                }
            }
        }

        pub async fn list_peer_descriptors(&mut self) -> Result<Vec<PeerDescriptor>, ClientError<ListPeerDescriptorsError>> {

            let request = tonic::Request::new(peer_manager::ListPeerDescriptorsRequest {});

            let response = self.inner.list_peer_descriptors(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::list_peer_descriptors_response::Reply::Failure(failure) => {
                    let error = ListPeerDescriptorsError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::list_peer_descriptors_response::Reply::Success(success) => {
                    Ok(success.peers.into_iter()
                        .map(PeerDescriptor::try_from)
                        .collect::<Result<Vec<_>, _>>()?
                    )
                }
            }
        }

        pub async fn get_peer_state(&mut self, peer_id: PeerId) -> Result<PeerState, ClientError<GetPeerStateError>> {

            let request = tonic::Request::new(peer_manager::GetPeerStateRequest {
                peer_id: Some(peer_id.into()),
            });

            let response = self.inner.get_peer_state(request).await?
                .into_inner();

            match extract!(response.reply)? {
                peer_manager::get_peer_state_response::Reply::Failure(failure) => {
                    let error = GetPeerStateError::try_from(failure)?;
                    Err(ClientError::UsageError(error))
                }
                peer_manager::get_peer_state_response::Reply::Success(success) => {
                    let peer_descriptor = extract!(success.state)?;
                    Ok(peer_descriptor)
                }
            }
        }
        
        pub async fn create_peer_setup(&mut self, peer_id: PeerId, user_id: String) -> Result<PeerSetup, CreateSetupError> {
            let request = tonic::Request::new(
                peer_manager::GeneratePeerSetupRequest {
                    peer: Some(peer_id.into()),
                    user_id,
                }
            );

            match self.inner.generate_peer_setup(request).await {
                Ok(response) => {
                    match response.into_inner().reply {
                        Some(peer_manager::generate_peer_setup_response::Reply::Success(peer_manager::GeneratePeerSetupSuccess { setup, .. })) => {
                            setup
                                .ok_or(CreateSetupError { message: format!("Failed to create setup-string for peer <{}>! Got no PeerSetup!", peer_id) })
                                .and_then(|setup| PeerSetup::try_from(setup)
                                    .map_err(|cause| CreateSetupError { message: cause.to_string() })
                                )
                        }
                        _ => {
                            let error = CreateSetupError { message: format!("Failed to create setup-string for peer <{}>!", peer_id) };
                            error!("{}", error);
                            Err(error)
                        }
                    }
                }
                Err(status) => {
                    Err(CreateSetupError { message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn create_cleo_setup(&mut self, user_id: String) -> Result<CleoSetup, CreateSetupError> {
            let request = tonic::Request::new(
                peer_manager::GenerateCleoSetupRequest {
                    user_id
                }
            );

            match self.inner.generate_cleo_setup(request).await {
                Ok(response) => {
                    match response.into_inner().reply {
                        Some(peer_manager::generate_cleo_setup_response::Reply::Success(peer_manager::GenerateCleoSetupSuccess { setup, .. })) => {
                            setup
                                .ok_or(CreateSetupError { message: "Failed to create setup-string for CLEO! Got no CleoSetup!".to_owned() })
                                .and_then(|setup| CleoSetup::try_from(setup)
                                    .map_err(|cause| CreateSetupError { message: cause.to_string() })
                                )
                        }
                        _ => {
                            let error = CreateSetupError { message: "Failed to create setup-string for CLEO!".to_owned() };
                            error!("{}", error);
                            Err(error)
                        }
                    }
                }
                Err(status) => {
                    Err(CreateSetupError { message: format!("gRPC failure: {status}") })
                }
            }
        }

        pub async fn list_devices(&mut self) -> Result<Vec<DeviceDescriptor>, ListDevicesError> {
            let request = tonic::Request::new(peer_manager::ListDevicesRequest {});

            match self.inner.list_devices(request).await {
                Ok(response) => {
                    response.into_inner().devices
                        .into_iter()
                        .map(DeviceDescriptor::try_from)
                        .collect::<Result<_, _>>()
                        .map_err(|cause| ListDevicesError::Internal { cause: cause.to_string() })
                },
                Err(status) => {
                    Err(ListDevicesError::Internal { cause: format!("gRPC failure: {status}") })
                },
            }
        }
    }

    #[derive(thiserror::Error, Debug)]
    #[error("{message}")]
    pub struct CreateSetupError {
        message: String,
    }
}
