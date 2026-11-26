pub mod components;

use leptos::prelude::*;
use crate::util::client_error_description;
use opendut_lea_components::{ErrorMessage, ErrorMessageType, ErrorText};
use opendut_model::peer::state::PeerConnectionState;
use crate::app::use_app_globals;
use crate::api::peers::list_peer_states;
use crate::api::clusters::list_cluster_descriptors;
use crate::clusters::ClustersCard;
use crate::peers::PeersCard;

#[derive(Clone)]
pub(crate) struct Peers {
    pub online: usize,
    pub offline: usize,
}

#[derive(Clone)]
pub(crate) struct Clusters {
    pub deployed: usize,
    pub undeployed: usize,
}

#[component]
pub fn Dashboard() -> impl IntoView {

    leptos::leptos_dom::helpers::document().set_title("openDuT");

    let globals = use_app_globals();
    let cluster_client = globals.client.clone();

    let peer_states = list_peer_states(None);
    let peers = Signal::derive(move || {
        peer_states.get().map(|result| result.map(|peer_states| {
            let (online, offline): (Vec<_>, Vec<_>) = peer_states.values()
                .partition(|peer_state| match peer_state.connection {
                    PeerConnectionState::Online { .. } => true,
                    PeerConnectionState::Offline => false,
                });

            Peers {
                offline: offline.len(),
                online: online.len(),
            }
        }))
    });

    let cluster_descriptors = list_cluster_descriptors();
    let cluster_deployments = LocalResource::new(move || {
        let mut carl = cluster_client.clone();
        async move {
            let deployed = carl.cluster.list_cluster_deployments().await
                .map_err(|error| {
                    let description = client_error_description(
                        &error,
                        "The cluster deployments could not be loaded. Please check your connection and try again.",
                    );
                    ErrorText {
                        headline: String::from("Unable to load cluster deployments"),
                        description
                    }
                })?
                .len();

            Ok(deployed)
        }
    });

    let clusters = Signal::derive(move || {
        match (cluster_descriptors.get(), cluster_deployments.get()) {
            (Some(Err(error)), _) | (_, Some(Err(error))) => Some(Err(error)),
            (Some(Ok(descriptors)), Some(Ok(deployed))) => Some(Ok(Clusters {
                deployed,
                undeployed: descriptors.len().saturating_sub(deployed),
            })),
            _ => None,
        }
    });

    view! {
        <div class="container is-fluid">
            { move || {
                if let Some(Err(error)) = peers.get() {
                    view! {
                        <ErrorMessage
                            text=error
                            on_retry=move || peer_states.refetch()
                            error_type=ErrorMessageType::Warning
                        />
                    }.into_any()
                } else { ().into_any() }
            }}
            { move || {
                if let Some(Err(error)) = clusters.get() {
                    view! {
                        <ErrorMessage
                            text=error
                            on_retry=move || {
                                cluster_descriptors.refetch();
                                cluster_deployments.refetch();
                            }
                            error_type=ErrorMessageType::Warning
                        />
                    }.into_any()
                } else { ().into_any() }
            }}
        </div>

        <div class="mt-6">
            <div class="columns is-full">
                <div class="column">
                    <h1 class="title is-3 has-text-centered">Welcome</h1>
                </div>
            </div>
            <div class="mt-5">
                <div class="columns is-centered">
                    <div class="column is-one-third">
                        <ClustersCard clusters />
                    </div>
                    <div class="column is-one-third">
                        <PeersCard peers />
                    </div>
                </div>
            </div>
        </div>
    }
}
