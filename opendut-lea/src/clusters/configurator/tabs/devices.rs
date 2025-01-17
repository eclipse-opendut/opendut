use leptos::prelude::*;
use opendut_types::peer::PeerDescriptor;

use crate::clusters::configurator::DeviceSelector;
use crate::clusters::configurator::types::UserClusterConfiguration;

#[component]
pub fn DevicesTab(
    cluster_configuration: RwSignal<UserClusterConfiguration>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {

    view! {
        <div>
            <DeviceSelector cluster_configuration peers />
        </div>
    }
}
