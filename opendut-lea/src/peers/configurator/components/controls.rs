use std::collections::HashSet;
use std::ops::Not;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_model::cluster::ClusterId;
use opendut_model::peer::PeerDescriptor;

use crate::app::use_app_globals;
use crate::components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use crate::peers::components::DeletePeerButton;
use crate::peers::configurator::types::UserPeerConfiguration;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(
    configuration: RwSignal<UserPeerConfiguration>,
    is_valid_peer_configuration: Signal<bool>,
) -> impl IntoView {

    let peer_id = Signal::derive(move || {
        configuration.get().id
    });

    let used_clusters_length = Signal::derive(move || {
        let mut used_clusters: HashSet<ClusterId> = HashSet::new();
        let _ = configuration.get().devices
            .into_iter()
            .filter(|device| device.get().contained_in_clusters.is_empty().not() )
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
        <div class="is-flex">
            <SavePeerButton
                configuration
                is_valid_peer_configuration
            />
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
    configuration: RwSignal<UserPeerConfiguration>,
    is_valid_peer_configuration: Signal<bool>,
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let setter = create_write_slice(
        configuration,
        |config, input| {
            config.is_new = input;
        },
    );

    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if pending.get() {
            ButtonState::Loading
        } else if is_valid_peer_configuration.get() {
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

            let peer_descriptor = PeerDescriptor::try_from(configuration.get_untracked());
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

    view! {
        <IconButton
            icon=FontAwesomeIcon::Save
            color=ButtonColor::Info
            size=ButtonSize::Normal
            state=button_state
            label="Save Peer"
            on_action
        />
    }
}
