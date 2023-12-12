use leptos::*;
use opendut_types::peer::{PeerDescriptor, PeerId};

use crate::api::use_carl;
use crate::components::{ButtonColor, ButtonState, ButtonStateSignalProvider, ConfirmationButton, FontAwesomeIcon, IconButton};
use crate::peers::configurator::types::UserPeerConfiguration;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {

    view! {
        <div class="buttons">
            <SavePeerButton configuration=configuration />
            <DeletePeerButton configuration=configuration />
        </div>
    }
}

#[component]
fn SavePeerButton(configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {

    let carl = use_carl();

    let store_action = create_action(move |_: &()| {
        async move {
            let mut carl = carl.get_untracked();
            let peer_descriptor = PeerDescriptor::try_from(configuration.get_untracked());
            match peer_descriptor {
                Ok(peer_descriptor) => {
                    let peer_id = peer_descriptor.id;
                    let result = carl.peers.create_peer(peer_descriptor).await;
                    match result {
                        Ok(_) => {
                            log::info!("Successfully create peer: {}", peer_id);
                        }
                        Err(cause) => {
                            log::error!("Failed to create peer <{peer_id}>, due to error: {cause:?}");
                        }
                    }
                }
                Err(_) => {
                    log::error!("Failed to dispatch create peer action, due to misconfiguration!");
                }
            }
        }
    });

    let button_state = MaybeSignal::derive(move || {
        if store_action.pending().get() {
            ButtonState::Loading
        }
        else {
            configuration.with(|configuration| {
                if configuration.is_valid() {
                    ButtonState::Enabled
                }
                else {
                    ButtonState::Disabled
                }
            })
        }
    });

    view! {
        <IconButton
            icon=FontAwesomeIcon::Save
            color=ButtonColor::Info
            state=button_state
            label="Save Peer"
            on_action=move || {
                store_action.dispatch(());
            }
        />
    }
}

#[component]
fn DeletePeerButton(configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {

    let carl = use_carl();

    let delete_action = create_action(move |_: &PeerId| {
        async move {
            let mut carl = carl.get_untracked();
            let peer_id = configuration.get_untracked().id;
            let result = carl.peers.delete_peer(peer_id).await;
            match result {
                Ok(_) => {
                    log::info!("Successfully deleted peer: {}", peer_id);
                    navigate_to(WellKnownRoutes::PeersOverview);
                }
                Err(cause) => {
                    log::error!("Failed to delete peer <{peer_id}>, due to error: {cause:?}");
                }
            }
        }
    });

    let button_state = delete_action
        .pending()
        .derive_loading();

    view! {
        <ConfirmationButton
            icon=FontAwesomeIcon::TrashCan
            color=ButtonColor::Danger
            state=button_state
            label="Remove Peer?"
            on_conform=move || {
                configuration.with_untracked(|config| {
                    delete_action.dispatch(config.id);
                });
            }
        />
    }
}
