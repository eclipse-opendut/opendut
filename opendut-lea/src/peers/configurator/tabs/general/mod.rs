use leptos::prelude::*;
use crate::components::ReadOnlyInput;
use crate::peers::configurator::components::{PeerLocationInput, PeerNameInput};
use crate::peers::configurator::types::UserPeerDescriptor;

#[component]
pub fn GeneralTab(user_peer_descriptor: RwSignal<UserPeerDescriptor>) -> impl IntoView {

    let peer_id = Signal::derive(move || user_peer_descriptor.get().id.to_string());

    view! {
        <div>
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
