use std::collections::HashSet;
use std::ops::Not;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::cluster::ClusterId;
use opendut_model::peer::PeerDescriptor;
use opendut_model::peer::state::PeerState;

use crate::app::use_app_globals;
use crate::components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use crate::peers::components::DeletePeerButton;
use crate::peers::configurator::types::UserPeerDescriptor;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::peers::components::PeerHealth;
use crate::util;

#[component]
pub fn Controls(
    user_peer_descriptor: RwSignal<UserPeerDescriptor>,
    peer_state: Signal<PeerState>
) -> impl IntoView {

    let peer_id = Signal::derive(move || {
        user_peer_descriptor.get().id
    });

    let used_clusters_length = Signal::derive(move || {
        let mut used_clusters: HashSet<ClusterId> = HashSet::new();
        let _ = user_peer_descriptor.get().devices
            .into_iter()
            .filter(|device| device.get().contained_in_clusters.is_empty().not())
            .map(|device| for cluster_descriptor in device.get().contained_in_clusters {
                used_clusters.insert(cluster_descriptor.id);
            })
            .collect::<Vec<_>>();

        used_clusters.len()
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
        navigate_to(WellKnownRoutes::PeersOverview, use_navigate.clone())
    }};

    view! {
        <div class="is-flex is-align-items-center">
            <PeerHealth state=peer_state />
            <div class="px-2" />
            <SavePeerButton user_peer_descriptor />
            <div class="px-1" />
            <DeletePeerButton
                peer_id
                used_clusters_length
                button_color=ButtonColor::Danger
                on_delete
            />
        </div>
    }
}

#[component]
fn SavePeerButton(
    user_peer_descriptor: RwSignal<UserPeerDescriptor>,
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let setter = create_write_slice(
        user_peer_descriptor,
        |descriptor, input| {
            descriptor.is_new = input;
        },
    );

    let all_tabs_valid = Memo::new(move |_| {
        user_peer_descriptor.with(|descriptor| descriptor.is_valid())
    });

    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if pending.get() {
            ButtonState::Loading
        } else if all_tabs_valid.get() {
            ButtonState::Enabled
        } else {
            ButtonState::Disabled
        }
    });

    let on_action = move || {
        let toaster = Arc::clone(&toaster);
        let mut carl = globals.client.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let peer_descriptor = PeerDescriptor::try_from(user_peer_descriptor.get_untracked());
            match peer_descriptor {
                Ok(peer_descriptor) => {
                    let peer_id = peer_descriptor.id;
                    let result = carl.peers.store_peer_descriptor(peer_descriptor).await;
                    match result {
                        Ok(_) => {
                            debug!("Successfully stored peer: {peer_id}");
                            toaster.toast(
                                Toast::builder()
                                    .simple("Successfully stored peer configuration.")
                                    .success(),
                            );
                            setter.set(false);
                        }
                        Err(cause) => {
                            error!("Failed to create peer <{peer_id}>, due to error: {cause:?}");
                            toaster.toast(Toast::builder().simple("Failed to store peer!").error());
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to dispatch create peer action, due to misconfiguration!\n  {error}");
                }
            };

            pending.set(false);
        })
    };

    let hide_tooltip = Signal::derive(move || {
        all_tabs_valid.get()
    });

    let tooltip_content = Box::new(move || {
        if !all_tabs_valid.get() {
            util::view::tooltip_content_for_configurator_errors("Peer")
        } else { ().into_any() }
    });

    view! {
        <Tooltip
            text=tooltip_content
            direction=TooltipDirection::Right
            is_hidden=hide_tooltip
        >
            <IconButton
                icon=FontAwesomeIcon::Save
                color=ButtonColor::Info
                size=ButtonSize::Normal
                state=button_state
                label="Save Peer"
                on_action
            />
        </Tooltip>
    }
}
