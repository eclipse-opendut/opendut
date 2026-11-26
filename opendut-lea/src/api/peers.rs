use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::peer::GetPeerDescriptorError;
use std::collections::HashMap;
use leptos::prelude::{Get, LocalResource, RwSignal, Signal, Track};
use tracing::error;
use opendut_lea_components::ErrorText;
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::peer::state::PeerState;
use crate::app::use_app_globals;
use crate::util::client_error_description;

pub fn list_peer_descriptors(
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<Vec<PeerDescriptor>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let mut peers = carl.peers.list_peer_descriptors().await
                .map_err(|error| {
                    error!("Failed to request the list of peers: {error}");

                    let error_description = client_error_description(
                        &error,
                        "The peer descriptors could not be loaded. Please check your connection and try again.",
                    );

                    ErrorText {
                        headline: String::from("Unable to load peer descriptors"),
                        description: error_description,
                    }
                })?;

            peers.sort_by(|peer_a, peer_b| {
                peer_a.name.value().to_lowercase()
                    .cmp(&peer_b.name.value().to_lowercase())
                    .then(peer_a.id.to_string().cmp(&peer_b.id.to_string()))
            });

            Ok(peers)
        }
    })
}

pub fn list_peer_states(refetch_signal: Option<RwSignal<()>>) -> LocalResource<Result<HashMap<PeerId, PeerState>, ErrorText>> {
    LocalResource::new(move || {
        refetch_signal.track();

        let globals = use_app_globals();
        let mut carl = globals.client;

        async move {
            carl.peers.list_peer_states().await
                .map_err(|error| {
                    error!("Failed to request the list of peer states: {error}");

                    let error_description = client_error_description(
                        &error,
                        "The peer states could not be loaded. Please check your connection and try again.",
                    );

                    ErrorText {
                        headline: String::from("Unable to load peer states"),
                        description: error_description,
                    }
                })
        }
    })
}
pub fn get_peer_descriptor(
    id: impl Into<Signal<PeerId>>,
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<Option<PeerDescriptor>, ErrorText>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            match carl.peers.get_peer_descriptor(id).await {
                Ok(descriptor) => Ok(Some(descriptor)),
                Err(ClientError::UsageError(GetPeerDescriptorError::PeerNotFound { .. })) => Ok(None),
                Err(error) => {
                    error!("Unable to load peer descriptor: {error}");
                    Err(ErrorText {
                        headline: String::from("Unable to load peer descriptor"),
                        description: client_error_description(
                            &error,
                            "The peer descriptor could not be loaded. Please check your connection and try again.",
                        ),
                    })
                }
            }
        }
    })
}

pub fn get_peer_state(
    id: impl Into<Signal<PeerId>>,
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<PeerState, ErrorText>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            carl.peers.get_peer_state(id).await.map_err(|error| {
                error!("Unable to load peer state: {error}");
                ErrorText {
                    headline: String::from("Unable to load peer state"),
                    description: client_error_description(
                        &error,
                        "The peer state could not be loaded. Please check your connection and try again.",
                    ),
                }
            })
        }
    })
}
