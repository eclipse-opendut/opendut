use crate::peers::components::DeletePeerButton;
use crate::util;
use leptos::html::Div;
use leptos::prelude::*;
use leptos::web_sys;
use leptos_router::hooks::use_navigate;
use leptos_use::on_click_outside;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use opendut_model::peer::state::{PeerState};
use crate::peers::components::PeerHealth;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    peer_descriptor: RwSignal<PeerDescriptor>,
    peer_state: RwSignal<PeerState>,
    cluster_descriptor: RwSignal<Vec<ClusterDescriptor>>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where
    OnDeleteFn: Fn() + Copy + Send + 'static,
{
    let peer_id = create_read_slice(peer_descriptor, |peer_descriptor| peer_descriptor.id);

    let peer_name = create_read_slice(peer_descriptor, |peer_descriptor| {
        Clone::clone(&peer_descriptor.name).to_string()
    });

    let configurator_href = move || format!("/peers/{}/configure/general", peer_id.get());
    let setup_href = move || format!("/peers/{}/configure/setup", peer_id.get());

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false));
    let used_clusters_length = RwSignal::new(0);

    let cluster_columns = move || {
        let devices_in_peer = peer_descriptor
            .get()
            .topology
            .devices
            .into_iter()
            .map(|device| device.id)
            .collect::<Vec<_>>();

        let devices_in_cluster = {
            let mut devices_in_cluster = cluster_descriptor
                .get()
                .into_iter()
                .map(|cluster| (cluster.id, cluster.name, cluster.devices))
                .collect::<Vec<(_, _, _)>>();

            devices_in_cluster.sort_by(|(_, name_a, _), (_, name_b, _)| {
                name_a.to_string().cmp(&name_b.to_string())
            });

            devices_in_cluster
        };

        let cluster_view_list: Vec<View<_>> = devices_in_cluster
            .into_iter()
            .filter(|(_, _, devices)| {
                devices_in_peer
                    .clone()
                    .into_iter()
                    .any(|device| devices.contains(&device))
            })
            .map(|(cluster_id, cluster_name, _)| {
                let cluster_name = move || cluster_name.to_string();
                let configurator_href = move || format!("/clusters/{cluster_id}/configure/general");
                view! {
                    <a href=configurator_href on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()> { cluster_name } </a>
                }
            })
            .collect();

        used_clusters_length.set(cluster_view_list.len());

        util::view::join_with_comma_spans(cluster_view_list)
    };

    let mousedown_pos = RwSignal::new((0, 0));
    let use_navigate = use_navigate();

    let on_mousedown = move |e: web_sys::MouseEvent| {
        mousedown_pos.set((e.client_x(), e.client_y()));
    };

    let on_click = move |e: web_sys::MouseEvent| {
        let distance = util::calculate_distance(mousedown_pos.get(), (e.client_x(), e.client_y()));
        // fixes text selection issue: mouse moved < threshold -> click, else it's a drag.
        if distance < util::MOUSE_DRAG_PIXEL_THRESHOLD {
            use_navigate(&configurator_href(), Default::default());
        }
    };

    view! {
        <tr
            class="is-clickable"
            on:mousedown=on_mousedown
            on:click=on_click
        >
            <td class="is-vcentered">
                <PeerHealth state=peer_state.into() />
            </td>
            <td class="is-vcentered">
                <a href=configurator_href on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()> { peer_name } </a>
            </td>
            <td class="is-vcentered">
                { cluster_columns }
            </td>
            <td class="is-vcentered">
                <div class="is-flex">
                    <DeletePeerButton
                        on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()
                        peer_id used_clusters_length
                        button_color=ButtonColor::TextDanger
                        on_delete
                    />
                    <div class="dropdown is-right pl-2" class=("is-active", move || dropdown_active.get())>
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
