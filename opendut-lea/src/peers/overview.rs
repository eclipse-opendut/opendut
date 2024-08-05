use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::cluster::ClusterConfiguration;
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

        let registered_peers: Resource<(), Vec<PeerDescriptor>> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.peers.list_peer_descriptors().await
                    .expect("Failed to request the list of peers.")
            }
        });

        let connected_peers: Resource<(), Vec<PeerId>> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.broker.list_peers().await
                    .expect("Failed to request the list of connected peers.")
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
            if let (Some(registered_peers), Some(connected_peers), Some(configured_clusters)) = (registered_peers.get(), connected_peers.get(), configured_clusters.get()) {
                registered_peers.into_iter().map(|peer_descriptor| {
                    let peer_id = peer_descriptor.id;
                    view! {
                        <Row
                            peer_descriptor=create_rw_signal(peer_descriptor)
                            cluster_configuration=create_rw_signal(configured_clusters.clone())
                            is_connected={connected_peers.contains(&peer_id)}
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
                                connected_peers.refetch();
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
                                    <th>"Used in Clusters"</th>
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
    cluster_configuration: RwSignal<Vec<ClusterConfiguration>>,
    is_connected: bool,
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
        let state = if is_connected {
            health::State {
                kind: health::StateKind::Green,
                text: String::from("Connected. No errors."),
            }
        }
        else {
            health::State {
                kind: health::StateKind::Unknown,
                text: String::from("Disconnected"),
            }
        };
        create_signal(state)
    };

    let dropdown_active = create_rw_signal(false);
    let dropdown = create_node_ref::<Div>();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    let cluster_columns = MaybeSignal::derive(move || {
        let devices_in_peer = peer_descriptor.get().topology.devices.into_iter().map(|device| device.id).collect::<Vec<_>>();
        let devices_in_cluster =  cluster_configuration.get().into_iter().map(|cluster| (cluster.id, cluster.name, cluster.devices)).collect::<Vec<(_,_,_)>>();
        
        let cluster_view_list = devices_in_cluster.into_iter()
            .filter(|(_, _, devices)| devices_in_peer.clone().into_iter().any(|device| devices.contains(&device)))
            .map(|(clusterId, clusterName, _)| {
                let cluster_name = move || { clusterName.to_string() };
                let configurator_href = move || { format!("/clusters/{}/configure/general", clusterId) };
                view! {
                    <span><a href={ configurator_href } >{ cluster_name }</a></span>
                }
            }).collect::<Vec<_>>();

        let cluster_view_list_length = cluster_view_list.len();
        let mut cluster_view_list_with_seperator: Vec<HtmlElement<_>> = Vec::new();
        for (i, e) in cluster_view_list.into_iter().enumerate() {
            cluster_view_list_with_seperator.push(e);
            if i < (cluster_view_list_length - 1) {
                cluster_view_list_with_seperator.push(
                    view! { <span>", "</span> }
                );
            }
        }
        cluster_view_list_with_seperator
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
