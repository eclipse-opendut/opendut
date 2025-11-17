mod row;

use crate::app::use_app_globals;
use crate::components::LoadingSpinner;
use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::peers::components::CreatePeerButton;
use leptos::prelude::*;
use tracing::trace;
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::state::PeerState;
use opendut_model::peer::PeerDescriptor;
use crate::peers::overview::row::Row;

#[component(transparent)]
pub fn PeersOverview() -> impl IntoView {

    let globals = use_app_globals();

    let refetch_registered_peers = RwSignal::new(());

    let registered_peers: LocalResource<Vec<(PeerDescriptor, PeerState)>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            refetch_registered_peers.track();

            let mut carl = carl.clone();
            async move {
                let peers = carl.peers.list_peer_descriptors().await
                    .expect("Failed to request the list of peers.");

                let mut peer_states = carl.peers.list_peer_states().await
                    .expect("Failed to request the list of peer states.");

                let mut peers_with_state: Vec<(PeerDescriptor, PeerState)> = vec![];
                for peer in peers {
                    let peer_state = peer_states.remove(&peer.id)
                        .unwrap_or_else(|| {
                            trace!("Did not receive PeerState for peer <{peer_id}>. Treating it as down.", peer_id=peer.id);
                            PeerState::default()
                        });
                    peers_with_state.push((peer, peer_state));
                }

                peers_with_state
            }
        })
    };

    let configured_clusters: LocalResource<Vec<ClusterDescriptor>> = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_descriptors().await
                    .expect("Failed to request the list of peers.")
            }
        })
    };

    let peers_table_rows = LocalResource::new(move || async move {
        let mut registered_peers = registered_peers.await;
        registered_peers.sort_by(|(peer_a, _), (peer_b, _)|
            peer_a.name.value().to_lowercase()
                .cmp(&peer_b.name.value().to_lowercase())
        );

        registered_peers
    });

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Peers", "/peers")
    ];

    view!{
        <BasePageContainer
            title="Peers"
            breadcrumbs=breadcrumbs
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
            <div class="mt-4">
                <Transition
                    fallback=LoadingSpinner
                >
                    {move || {
                        Suspend::new(async move {
                            let peers_table_rows = peers_table_rows.await;
                            let configured_clusters = configured_clusters.await;

                            view! {
                                <table class="table is-hoverable is-fullwidth">
                                    <thead>
                                        <tr>
                                            <th class="is-narrow">"Health"</th>
                                            <th>"Name"</th>
                                            <th>"Configured in Clusters"</th>
                                            <th class="is-narrow has-text-centered">"Action"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each = move || peers_table_rows.clone()
                                            key = |(peer, _)| peer.id
                                            children = { move |(peer_descriptor, peer_state)| {
                                                let on_delete = move || {
                                                    refetch_registered_peers.notify();
                                                };
                                                view! {
                                                    <Row
                                                        peer_descriptor=RwSignal::new(peer_descriptor)
                                                        peer_state=RwSignal::new(peer_state)
                                                        cluster_descriptor=RwSignal::new(configured_clusters.clone())
                                                        on_delete
                                                    />
                                                }
                                            }}
                                        />
                                    </tbody>
                                </table>
                            }
                        })
                    }}
                </Transition>
            </div>
        </BasePageContainer>
    }
}
