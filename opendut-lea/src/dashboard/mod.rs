pub mod components;

use leptos::prelude::*;
use opendut_lea_components::{ErrorMessage, ErrorMessageType, ErrorText};
use opendut_model::peer::state::PeerConnectionState;
use crate::app::use_app_globals;
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

    let peers: LocalResource<Result<Peers, ErrorText>> = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        async move {
            let peer_states = carl.peers.list_peer_states().await
                .map_err(|error| {
                    ErrorText {
                        headline: String::from("Unable to load peer states"),
                        description: error.to_string()
                    }
                })?;

            let (online, offline): (Vec<_>, Vec<_>) = peer_states.values()
                .partition(|peer_state| match peer_state.connection {
                    PeerConnectionState::Online { .. } => true,
                    PeerConnectionState::Offline => false,
                });

            Ok(Peers {
                offline: offline.len(),
                online: online.len(),
            })
        }
    });

    let clusters: LocalResource<Result<Clusters, ErrorText>> = LocalResource::new(move || {
        let mut carl = cluster_client.clone();
        async move {
            let configured = carl.cluster.list_cluster_descriptors().await
                .map_err(|error| {
                    ErrorText {
                        headline: String::from("Unable to load cluster descriptors"),
                        description: error.to_string()
                    }
                })?
                .len();

            let deployed = carl.cluster.list_cluster_deployments().await
                .map_err(|error| {
                    ErrorText {
                        headline: String::from("Unable to load cluster deployments"),
                        description: error.to_string()
                    }
                })?
                .len();

            Ok(Clusters {
                deployed,
                undeployed: configured.saturating_sub(deployed)
            })
        }
    });

    view! {
        { move || {
            if let Some(Err(error)) = peers.get() {
                view! {
                    <ErrorMessage
                        text=error
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
                        error_type=ErrorMessageType::Warning
                    />
                }.into_any()
            } else { ().into_any() }
        }}

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
