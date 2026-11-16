mod row;

use std::collections::HashMap;
use crate::app::use_app_globals;
use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::peers::components::CreatePeerButton;
use leptos::prelude::*;
use tracing::{error, trace};
use opendut_lea_components::{InfoMessage, ErrorMessage, ErrorMessageTableRow, ErrorMessageType, ErrorText, OverviewTable, TableHeading};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::state::PeerState;
use opendut_model::peer::{PeerDescriptor, PeerId};
use crate::peers::overview::row::Row;

#[component(transparent)]
pub fn PeersOverview() -> impl IntoView {

    let globals = use_app_globals();
    let carl = globals.client;

    let refetch_registered_peers = RwSignal::new(());

    let peer_states: LocalResource<Result<HashMap<PeerId, PeerState>, ErrorText>> = {
        let carl = carl.clone();

        LocalResource::new(move || {
            refetch_registered_peers.track();

            let mut carl = carl.clone();
            async move {
                carl.peers.list_peer_states().await
                    .map_err(|error| {
                        error!("Failed to request the list of peer states: {error}");

                        ErrorText {
                            headline: String::from("Unable to load peer states"),
                            description: String::from("The peer states could not be loaded. Please check your connection and try again."),
                        }
                    })
            }
        })
    };

    let peers: LocalResource<Result<Vec<PeerDescriptor>, ErrorText>> = {
        let carl = carl.clone();

        LocalResource::new(move || {
            refetch_registered_peers.track();
            
            let mut carl = carl.clone();
            async move {
                let mut peers = carl.peers.list_peer_descriptors().await
                    .map_err(|error| {
                        error!("Failed to request the list of peers: {error}");

                        ErrorText {
                            headline: String::from("Unable to load peer descriptors"),
                            description: String::from("The peer descriptors could not be loaded. Please check your connection and try again."),
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
    };

    let configured_clusters: LocalResource<Result<Vec<ClusterDescriptor>, ErrorText>> = {
        let carl = carl.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                let clusters = carl.cluster.list_cluster_descriptors().await
                    .map_err(|error| {
                        error!("Failed to request the list of clusters: {error}");

                        ErrorText {
                            headline: String::from("Unable to load cluster descriptors"),
                            description: String::from("The cluster descriptors could not be loaded. Please check your connection and try again."),
                        }
                    })?;

                Ok(clusters)
            }
        })
    };
    
    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Peers", "/peers")
    ];

    let table_headings = vec![
        TableHeading::new(String::from("Health")).set_narrow(),
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("Configured in Clusters")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

    view! {
        <BasePageContainer
            title="Peers"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreatePeerButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of peers"
                        on_action=move || {
                            refetch_registered_peers.notify();
                        }
                    />
                </div>
            }
        >
        <InfoMessage
            text="A Peer is a participant in a Cluster. \
                It is a computer on which EDGAR is installed and a Device you want to connect with others, \
                like an ECU, a restbus simulation or a test harness."
        />
        <Transition fallback=|| ()>
                { move || {
                    if let Some(Err(error)) = peer_states.get() {
                        view! {
                            <ErrorMessage
                                text=error
                                error_type=ErrorMessageType::Warning
                            />
                        }.into_any()
                    } else { ().into_any() }
                }}
                { move || {
                        if let Some(Err(error)) = configured_clusters.get() {
                        view! {
                            <ErrorMessage
                                text=error
                                error_type=ErrorMessageType::Warning
                            />
                        }.into_any()
                    } else { ().into_any() }
                }}
            </Transition>
        <OverviewTable headings=table_headings>
            { move || Suspend::new(async move {
                let peers = peers.await;
                let configured_clusters = configured_clusters.await;

                match peers {
                    Ok(peers) => {
                        view! {
                            <PeerOverviewTable
                                peers
                                peer_states
                                refetch_registered_peers
                                configured_clusters
                            />
                        }.into_any()
                    }
                    Err(error) => {
                        view! {
                            <ErrorMessageTableRow
                                text=error
                            />
                        }.into_any()
                    }
                }
            })}
            </OverviewTable>
        </BasePageContainer>
    }
}

#[component]
fn PeerOverviewTable(
    peers: Vec<PeerDescriptor>,
    peer_states: LocalResource<Result<HashMap<PeerId, PeerState>, ErrorText>>,
    refetch_registered_peers: RwSignal<()>,
    configured_clusters: Result<Vec<ClusterDescriptor>, ErrorText>,
) -> impl IntoView {

    if peers.is_empty() {
        view! {
            <tr>
                <td colspan="999" class="has-text-centered">"No peers have been created yet"</td>
            </tr>
        }.into_any()
    } else {
        view! {
            <For
                each = move || peers.clone()
                key = |peer| peer.id
                children = { move |peer_descriptor: PeerDescriptor| {
                    let peer_id = peer_descriptor.id;
                    let peer_state = Signal::derive(move || {
                        peer_states.with(|states| match states {
                            Some(Ok(states)) => Some(states.get(&peer_id).cloned().unwrap_or_else(|| {
                                trace!("Did not receive PeerState for peer <{peer_id}>. Treating it as down.");
                                PeerState::default()
                            })),
                            Some(Err(_)) | None => None,
                        })
                    });
                    let on_delete = move || {
                        refetch_registered_peers.notify();
                    };
                    view! {
                        <Row
                            peer_descriptor=RwSignal::new(peer_descriptor)
                            peer_state
                            cluster_descriptors=RwSignal::new(configured_clusters.clone())
                            on_delete
                        />
                    }
                }}
            />
        }.into_any()
    }
}
