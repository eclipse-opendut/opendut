use leptos::{component, IntoView, MaybeSignal, RwSignal, SignalGet, view};

use crate::components::ReadOnlyInput;
use crate::peers::configurator::components::PeerNameInput;
use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn GeneralTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {

    let peer_id = MaybeSignal::derive(move || peer_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Peer ID"
                value=peer_id
            />
            <PeerNameInput
                peer_configuration=peer_configuration
            />
        </div>
    }
}
