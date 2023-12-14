use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;

use opendut_types::peer::PeerId;

use crate::app::{ExpectGlobals, use_app_globals};
use crate::components::{BasePageContainer, IconButton, ButtonColor, ButtonState, FontAwesomeIcon, Breadcrumb, Initialized, ButtonSize};
use crate::components::health;
use crate::components::health::Health;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component(transparent)]
pub fn PeersOverview() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

        let globals = use_app_globals();

        let registered_peers: Resource<(), Vec<PeerId>> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.peers.list_peer_descriptors().await
                    .expect("Failed to request the list of peers.")
                    .iter().map(|peer| peer.id) // TODO: Don't discard the other information.
                    .collect::<Vec<_>>()
            }
        });

        let connected_peers: Resource<(), Vec<PeerId>> = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.broker.list_peers().await
                    .expect("Failed to request the list of connected peers.")
            }
        });

        let remove_peer = create_action(move |id: &PeerId| {
            let mut carl = globals.expect_client();
            let id = Clone::clone(id);
            async move {
                let _ = carl.peers.delete_peer_descriptor(id).await;
                registered_peers.refetch();
            }
        });

        let peers_table_rows = move || {

            if let (Some(registered_peers), Some(connected_peers)) = (registered_peers.get(), connected_peers.get()) {
                registered_peers.into_iter().map(|peer| {
                    view! {
                        <Row
                            id={peer}
                            is_connected={connected_peers.contains(&peer)}
                            on_remove=move || remove_peer.dispatch(peer)
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
                        <IconButton
                            icon=FontAwesomeIcon::Plus
                            color=ButtonColor::Success
                            size=ButtonSize::Normal
                            state=ButtonState::Enabled
                            label="Create peer"
                            on_action=move || {
                                navigate_to(WellKnownRoutes::PeerConfigurator {
                                    id: PeerId::random()
                                });
                            }
                        />
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
                                    <th>"ID"</th>
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
fn Row<OnRemove>(
    id: PeerId,
    is_connected: bool,
    on_remove: OnRemove
) -> impl IntoView
where OnRemove: Fn() + 'static {

    let formatted_id = move || { format!("{}", id) };
    let edge_href = move || { format!("/peers/{}/configure/general", id) };

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

    view! {
        <tr>
            <td class="is-vcentered">
                <Health state=health_state />
            </td>
            <td class="is-vcentered">
                <a href={ edge_href() } >{ formatted_id() }</a>
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
                                    href={ format!("/peers/{}/configure/setup", id) }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-download"></i>
                                    </span>
                                    <span>"Setup"</span>
                                </a>
                                <button
                                    class="button is-white is-fullwidth is-justify-content-flex-start"
                                    aria-label="Remove Peer"
                                    on:click=move |_| {
                                        dropdown_active.set(false);
                                        on_remove();
                                    }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-trash-can has-text-danger"></i>
                                    </span>
                                    <span>"Remove Peer"</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
        </tr>
    }
}
