use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::components::ReadOnlyInput;
use crate::peers::configurator::components::{PeerLocationInput, PeerNameInput};
use crate::peers::configurator::types::UserPeerDescriptor;

#[component]
pub fn GeneralTab(user_peer_descriptor: RwSignal<UserPeerDescriptor>) -> impl IntoView {

    let peer_id = Signal::derive(move || user_peer_descriptor.get().id.to_string());

    view! {
        <div>
            <InfoMessage
                text="A Peer is a participant in a Cluster. \
                    It is comprised of a computer on which EDGAR is installed and a Device you want to connect with others, \
                    like an ECU, a Restbus-Simulation or a test engine."
            />
            <ReadOnlyInput
                label="ID"
                value=peer_id
            />
            <PeerNameInput
                user_peer_descriptor
            />
            <PeerLocationInput
                user_peer_descriptor
            />
        </div>
    }
}
