use crate::peers::components::DeletePeerButton;
use crate::util;
use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::on_click_outside;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTableCell};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use opendut_model::peer::state::{PeerState};
use crate::components::ClickableOverviewTableRow;
use crate::peers::components::PeerHealth;

#[component]
pub(crate) fn Row<OnDeleteFn>(
    peer_descriptor: RwSignal<PeerDescriptor>,
    peer_state: RwSignal<PeerState>,
    cluster_descriptors: RwSignal<Vec<ClusterDescriptor>>,
    on_delete: OnDeleteFn,
) -> impl IntoView
where
    OnDeleteFn: Fn() + Copy + Send + 'static,
{
    let peer_id = create_read_slice(peer_descriptor, |peer_descriptor| peer_descriptor.id);

    let peer_name = create_read_slice(peer_descriptor, |peer_descriptor| {
        Clone::clone(&peer_descriptor.name).to_string()
    });

    let configurator_href = Signal::derive(move || format!("/peers/{}/configure/general", peer_id.get()));
    let setup_href = move || format!("/peers/{}/configure/setup", peer_id.get());

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();
    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false));

    let used_clusters_length = RwSignal::new(0);

    let cluster_column = move || {
        let devices_in_peer = peer_descriptor.get()
            .topology
            .devices
            .iter()
            .map(|device| device.id)
            .collect::<Vec<_>>();
        let configured_clusters_for_peer = util::list_configured_clusters_for_peer(devices_in_peer, cluster_descriptors.get());

        let cluster_view_list: Vec<View<_>> = configured_clusters_for_peer.into_iter()
            .map(|ClusterDescriptor { id, name, .. }| {
                let name = move || name.to_string();
                let configurator_href = move || format!("/clusters/{id}/configure/general");
                view! {
                    <a href=configurator_href on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()> { name } </a>
                }
            })
            .collect();

        used_clusters_length.set(cluster_view_list.len());

        util::view::join_with_comma_spans(cluster_view_list)
    };

    view! {
        <ClickableOverviewTableRow configurator_href>
            <OverviewTableCell>
                <PeerHealth state=peer_state />
            </OverviewTableCell>

            <OverviewTableCell>
                <a href=configurator_href on:click=|e| e.stop_propagation() on:mousedown=|e| e.stop_propagation()> { peer_name } </a>
            </OverviewTableCell>

            <OverviewTableCell>
                { cluster_column }
            </OverviewTableCell>

            <OverviewTableCell>
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
            </OverviewTableCell>
        </ClickableOverviewTableRow>
    }
}
