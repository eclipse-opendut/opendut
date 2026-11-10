use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
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
            <InfoMessage text="The Devices of individual Peers to connect into the Cluster." />
            <DeviceSelector user_cluster_descriptor peers is_disabled />
        </div>
    }
}
