use leptos::prelude::*;

use crate::components::{GenerateSetupStringForm, GenerateSetupStringKind, WarningMessage};
use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn SetupTab(peer_configuration: ReadSignal<UserPeerConfiguration>) -> impl IntoView {

    view! {
        <div class="field">
            <GenerateSetupStringForm kind=GenerateSetupStringKind::Edgar(peer_configuration.get().id) />
            <WarningMessage>"Setup-Strings may only be used to set up one host. For setting up multiple hosts, you should create a peer for each host."</WarningMessage>
        </div>
    }
}
