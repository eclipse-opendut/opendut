use leptos::{component, IntoView, RwSignal, view};

use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn DevicesTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {

    let _ = peer_configuration; // just to avoid unused warning

    view! {
        <div>
            "Devices"
        </div>
    }
}
