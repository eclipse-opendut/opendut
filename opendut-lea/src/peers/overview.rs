use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use opendut_types::peer::{PeerDescriptor};
use opendut_types::cluster::ClusterConfiguration;
use opendut_types::peer::state::PeerState;
use crate::app::{ExpectGlobals, use_app_globals};
use crate::components::{BasePageContainer, IconButton, ButtonColor, ButtonState, FontAwesomeIcon, Breadcrumb, ButtonSize, Initialized};
use crate::components::health;
use crate::components::health::Health;
use crate::peers::components::CreatePeerButton;

#[component(transparent)]
pub fn PeersOverview() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

        let globals = use_app_globals();

        let registered_peers: Resource<(), Vec<(PeerDescriptor, PeerState)>> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                let peers = carl.peers.list_peer_descriptors().await
                    .expect("Failed to request the list of peers.");
                
                let mut peers_with_state: Vec<(PeerDescriptor, PeerState)> = vec![];
                for peer in peers {
                    let peer_state = carl.peers.get_peer_state(peer.id).await.expect("Failed to request state of peer.");
                    peers_with_state.push((peer, peer_state));
                };

                peers_with_state
            }
        });

        let configured_clusters: Resource<(), Vec<ClusterConfiguration>> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.cluster.list_cluster_configurations().await
                    .expect("Failed to request the list of peers.")
            }
        });
        
        let peers_table_rows = move || {
            if let (Some(registered_peers), Some(configured_clusters)) = (registered_peers.get(), configured_clusters.get()) {
                registered_peers.into_iter().map(|(peer_descriptor, peer_state)| {
                    view! {
                        <Row
                            peer_descriptor=create_rw_signal(peer_descriptor)
                            peer_state=create_rw_signal(peer_state)
                            cluster_configuration=create_rw_signal(configured_clusters.clone())
                        />
                    }
                }).collect::<Vec<_>>()
            }
            else {
                Vec::new()
            }
        };

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
                                registered_peers.refetch();
                            }
                        />
                    </div>
                }
            >
                <div class="mt-4">
                    <Transition
                        fallback=move || view! { <p>"Loading..."</p> }
                    >
                        <table class="table is-hoverable is-fullwidth">
                            <thead>
                                <tr>
                                    <th class="is-narrow">"Health"</th>
                                    <th>"Name"</th>
                                    <th>"Configured in Clusters"</th>
                                    <th class="is-narrow">"Action"</th>
                                </tr>
                            </thead>
                            <tbody>
                                { peers_table_rows() }
                            </tbody>
                        </table>
                    </Transition>
                </div>
            </BasePageContainer>
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}

#[component]
fn Row(
    peer_descriptor: RwSignal<PeerDescriptor>,
    peer_state: RwSignal<PeerState>,
    cluster_configuration: RwSignal<Vec<ClusterConfiguration>>,
) -> impl IntoView {

    let peer_id = create_read_slice(peer_descriptor,
        |peer_descriptor| {
            peer_descriptor.id
        }
    );

    let peer_name = create_read_slice(peer_descriptor,
        |peer_descriptor| {
            Clone::clone(&peer_descriptor.name).to_string()
        }
    );

    let configurator_href = move || { format!("/peers/{}/configure/general", peer_id.get()) };
    let setup_href = move || { format!("/peers/{}/configure/setup", peer_id.get()) };

    let (health_state, _) = {
        let state = match peer_state.get() {
            PeerState::Down => {
                health::State {
                    kind: health::StateKind::Unknown,
                    text: String::from("Disconnected"),
                }
            }
            PeerState::Up { .. } => {
                health::State {
                    kind: health::StateKind::Green,
                    text: String::from("Connected. No errors."),
                }
            }
        };
        create_signal(state)
    };

    let dropdown_active = create_rw_signal(false);
    let dropdown = create_node_ref::<Div>();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    let cluster_columns = MaybeSignal::derive(move || {
        let devices_in_peer = peer_descriptor.get().topology.devices.into_iter().map(|device| device.id).collect::<Vec<_>>();

        let devices_in_cluster = {
            let mut devices_in_cluster = cluster_configuration.get().into_iter()
                .map(|cluster| (cluster.id, cluster.name, cluster.devices))
                .collect::<Vec<(_,_,_)>>();

            devices_in_cluster.sort_by(|(_, name_a, _), (_, name_b, _)|
                name_a.to_string().cmp(&name_b.to_string())
            );
            devices_in_cluster
        };

        let cluster_view_list: Vec<View> = devices_in_cluster.into_iter()
            .filter(|(_, _, devices)| devices_in_peer.clone().into_iter().any(|device| devices.contains(&device)))
            .map(|(clusterId, clusterName, _)| {
                let cluster_name = move || { clusterName.to_string() };
                let configurator_href = move || { format!("/clusters/{}/configure/general", clusterId) };
                view! {
                    <a href={ configurator_href }>{ cluster_name }</a>
                }.into_view()
            }).collect();

        join_with_comma_spans(cluster_view_list)
    });

    view! {
        <tr>
            <td class="is-vcentered">
                <Health state=health_state />
            </td>
            <td class="is-vcentered">
                <a href={ configurator_href } >{ peer_name }</a>
            </td>
            <td class="is-vcentered">
                { cluster_columns }
            </td>
            <td class="is-vcentered">
                <div class="is-pulled-right">
                    <div class="dropdown is-right" class=("is-active", move || dropdown_active.get())>
                        <div class="dropdown-trigger">
                            <IconButton
                                icon=FontAwesomeIcon::EllipsisVertical
                                color=ButtonColor::White
                                size=ButtonSize::Normal
                                state=ButtonState::Enabled
                                label="Show Peer Action Menu"
                                on_action=move || {
                                    dropdown_active.update(|value| *value = !*value);
                                }
                            />
                        </div>
                        <div node_ref=dropdown class="dropdown-menu">
                            <div class="dropdown-content">
                                <a
                                    class="button is-white is-fullwidth is-justify-content-flex-start"
                                    aria-label="Setup"
                                    href={ setup_href }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-download"></i>
                                    </span>
                                    <span>"Setup"</span>
                                </a>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
        </tr>
    }
}

fn join_with_comma_spans(elements: Vec<View>) -> Vec<View> {
    let elements_length = elements.len();

    let mut elements_with_separator = Vec::new();

    for (index, element) in elements.into_iter().enumerate() {
        elements_with_separator.push(element);

        if index < (elements_length - 1) {
            elements_with_separator.push(
                view! { <span>", "</span> }
                    .into_view()
            );
        }
    }
    elements_with_separator
}
