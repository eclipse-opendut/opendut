use std::rc::Rc;

use leptos::*;
use tracing::{debug, error, info};

use opendut_types::peer::{PeerDescriptor, PeerId};

use crate::app::{use_app_globals, ExpectGlobals};
use crate::components::{
    use_toaster, ButtonColor, ButtonSize, ButtonState, ButtonStateSignalProvider,
    ConfirmationButton, FontAwesomeIcon, IconButton, Toast,
};
use crate::peers::configurator::types::UserPeerConfiguration;
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

    let store_action = create_action(move |_: &()| {
        let toaster = Rc::clone(&toaster);
        async move {
            let mut carl = globals.expect_client();
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
            }
        }
    });

    let button_state = MaybeSignal::derive(move || {
        if store_action.pending().get() {
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
            on_action=move || {
                store_action.dispatch(());
            }
        />
    }
}

#[component]
fn DeletePeerButton(configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {
    let globals = use_app_globals();

    let delete_action = create_action(move |_: &PeerId| async move {
        let mut carl = globals.expect_client();
        let peer_id = configuration.get_untracked().id;
        let result = carl.peers.delete_peer_descriptor(peer_id).await;
        match result {
            Ok(_) => {
                info!("Successfully deleted peer: {}", peer_id);
                navigate_to(WellKnownRoutes::PeersOverview);
            }
            Err(cause) => {
                error!("Failed to delete peer <{peer_id}>, due to error: {cause:?}");
            }
        }
    });

    let button_state = delete_action.pending().derive_loading();

    view! {
        <ConfirmationButton
            icon=FontAwesomeIcon::TrashCan
            color=ButtonColor::Danger
            size=ButtonSize::Normal
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
