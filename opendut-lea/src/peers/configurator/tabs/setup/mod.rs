use leptos::either::Either;
use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::components::WarningMessage;
use crate::components::{GenerateSetupStringForm, GenerateSetupStringKind};
use crate::peers::configurator::types::UserPeerDescriptor;

#[component]
pub fn SetupTab(peer_configuration: ReadSignal<UserPeerDescriptor>) -> impl IntoView {

    let is_new = Signal::derive(move || peer_configuration.get().is_new);
    let kind = Signal::derive(move || GenerateSetupStringKind::Edgar(peer_configuration.get().id));

    view! {
        <InfoMessage
            text="Use the following string when setting up EDGAR on the target computer:"
        />
        <div class="field">
            {move || {
                if is_new.get() {
                    Either::Left(view! {
                        <WarningMessage
                            text="The peer configuration must be saved before a Setup-String can be generated. Please complete the configuration and save the peer first."
                        />
                    })
                } else {
                    Either::Right(view! {
                        <GenerateSetupStringForm kind />
                        <WarningMessage
                            text="Setup-Strings may only be used to set up one host. For setting up multiple hosts, you should create a peer for each host."
                        />
                    })
                }
            }}
        </div>
    }
}
