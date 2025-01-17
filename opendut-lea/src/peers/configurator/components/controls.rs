use std::collections::HashSet;
use std::ops::Not;
use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error, info};
use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerDescriptor;
use crate::app::use_app_globals;
use crate::components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ButtonStateSignalProvider, ConfirmationButton, DoorhangerButton, FontAwesomeIcon, IconButton, Toast};
use crate::peers::configurator::types::UserPeerConfiguration;
use crate::routing;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(
    configuration: RwSignal<UserPeerConfiguration>,
    is_valid_peer_configuration: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="buttons">
            <SavePeerButton configuration is_valid_peer_configuration />
            <DeletePeerButton configuration=configuration.read_only() />
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

    let button_state = Signal::derive(move || {
        if pending.get() {
            ButtonState::Loading
        } else if is_valid_peer_configuration.get() {
            ButtonState::Enabled
        } else {
            ButtonState::Disabled
        }
    });

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

#[component]
fn DeletePeerButton(configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {
    let globals = use_app_globals();
    let use_navigate = use_navigate();

    let pending = RwSignal::new(false);

    let on_conform = move || {
        let use_navigate = use_navigate.clone();
        let mut carl = globals.client.clone();
        let peer_id = configuration.get_untracked().id;

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.peers.delete_peer_descriptor(peer_id).await;
            match result {
                Ok(_) => {
                    info!("Successfully deleted peer: {}", peer_id);
                    navigate_to(WellKnownRoutes::PeersOverview, use_navigate);
                }
                Err(cause) => {
                    error!("Failed to delete peer <{peer_id}>, due to error: {cause:?}");
                }
            }

            pending.set(false);
        });
    };

    let button_state = Signal::from(pending).derive_loading();


    let delete_button = move || {
        let on_conform = on_conform.clone();

        let mut used_clusters: HashSet<ClusterId> = HashSet::new();
        let _ = configuration.get().devices
            .into_iter()
            .filter(|device| device.get().contained_in_clusters.is_empty().not() )
            .map(|device| for cluster_configuration in device.get().contained_in_clusters {
                used_clusters.insert(cluster_configuration.id);
            })
            .collect::<Vec<_>>();

        let used_clusters_length = used_clusters.len();

        if used_clusters_length > 0 {
            view! {
                <DoorhangerButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Danger
                    size=ButtonSize::Normal
                    state=button_state
                    label="Remove Peer?"
                >
                    <div style="white-space: nowrap">
                        "Peer can not be removed while it is configured in "{used_clusters_length}
                        <a class="has-text-link" href=routing::path::clusters_overview>" cluster(s)"</a>
                    </div>
                </DoorhangerButton>
            }.into_any()
        } else {
            view! {
                <ConfirmationButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Danger
                    size=ButtonSize::Normal
                    state=button_state
                    label="Remove Peer?"
                    on_conform
                />
            }.into_any()
        }
    };

    view! {
        <div>
            { delete_button }
        </div>
    }
}
