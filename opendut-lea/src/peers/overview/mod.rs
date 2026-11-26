mod row;

use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::peers::components::CreatePeerButton;
use leptos::prelude::*;
use opendut_lea_components::{InfoMessage, ErrorMessage, ErrorMessageTableRow, ErrorMessageType, ErrorText, OverviewTable, TableHeading};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use crate::api::clusters::list_cluster_descriptors;
use crate::api::peers::{list_peer_descriptors, list_peer_states};
use crate::peers::overview::row::Row;

#[component(transparent)]
pub fn PeersOverview() -> impl IntoView {

    let refetch_registered_peers = RwSignal::new(());

    let peer_states = list_peer_states(Some(refetch_registered_peers.into()));
    let peers = list_peer_descriptors(Some(refetch_registered_peers.into()));
    let configured_clusters = list_cluster_descriptors(Some(refetch_registered_peers.into()));
    
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
                                on_retry=move || peer_states.refetch()
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
                                on_retry=move || configured_clusters.refetch()
                                error_type=ErrorMessageType::Warning
                            />
                        }.into_any()
                    } else { ().into_any() }
                }}
            </Transition>
        <OverviewTable headings=table_headings>
            { move || Suspend::new(async move {
                let peers_result = peers.await;
                let configured_clusters_result = configured_clusters.await;

                match peers_result {
                    Ok(peers) => {
                        view! {
                            <PeerOverviewTable
                                peers
                                refetch_registered_peers
                                configured_clusters=configured_clusters_result
                            />
                        }.into_any()
                    }
                    Err(error) => {
                        view! {
                            <ErrorMessageTableRow
                                text=error
                                on_retry=move || peers.refetch()
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
                    let on_delete = move || {
                        refetch_registered_peers.notify();
                    };
                    view! {
                        <Row
                            peer_descriptor=RwSignal::new(peer_descriptor)
                            cluster_descriptors=RwSignal::new(configured_clusters.clone())
                            on_delete
                        />
                    }
                }}
            />
        }.into_any()
    }
}
