use leptos::prelude::*;
use opendut_types::peer::PeerDescriptor;

use crate::clusters::configurator::DeviceSelector;
use crate::clusters::configurator::types::UserClusterDescriptor;

#[component]
pub fn DevicesTab(
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {

    view! {
        <div>
            <DeviceSelector cluster_descriptor peers />
        </div>
    }
}
