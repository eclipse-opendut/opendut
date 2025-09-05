use leptos::prelude::*;

use crate::components::WarningMessage;
use crate::components::{GenerateSetupStringForm, GenerateSetupStringKind};
use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn SetupTab(peer_configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {

    let kind = Signal::derive(move || GenerateSetupStringKind::Edgar(peer_configuration.get().id));

    view! {
        <div class="field">
            <GenerateSetupStringForm kind />
            <WarningMessage>"Setup-Strings may only be used to set up one host. For setting up multiple hosts, you should create a peer for each host."</WarningMessage>
        </div>
    }
}
