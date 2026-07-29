use leptos::either::Either;
use leptos::prelude::*;

use crate::components::WarningMessage;
use crate::components::{GenerateSetupStringForm, GenerateSetupStringKind};
use crate::peers::configurator::types::UserPeerDescriptor;

#[component]
pub fn SetupTab(peer_configuration: ReadSignal<UserPeerDescriptor>) -> impl IntoView {

    let is_new = Signal::derive(move || peer_configuration.get().is_new);
    let kind = Signal::derive(move || GenerateSetupStringKind::Edgar(peer_configuration.get().id));

    view! {
        <div class="field">
            {move || {
                if is_new.get() {
                    Either::Left(view! {
                        <WarningMessage>
                            "The peer configuration must be saved before a Setup-String can be generated. Please complete the configuration and save the peer first."
                        </WarningMessage>
                    })
                } else {
                    Either::Right(view! {
                        <GenerateSetupStringForm kind />
                        <WarningMessage>
                            "Setup-Strings may only be used to set up one host. For setting up multiple hosts, you should create a peer for each host."
                        </WarningMessage>
                    })
                }
            }}
        </div>
    }
}