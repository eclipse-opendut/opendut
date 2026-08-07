use leptos::prelude::*;
use opendut_model::peer::PeerDescriptor;

use crate::clusters::configurator::DeviceSelector;
use crate::clusters::configurator::types::UserClusterDescriptor;

#[component]
pub fn DevicesTab(
    user_cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
    is_disabled: Signal<bool>,
) -> impl IntoView {

    view! {
        <div>
            <DeviceSelector user_cluster_descriptor peers is_disabled />
        </div>
    }
}
