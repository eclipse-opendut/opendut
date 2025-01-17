use leptos::prelude::*;
use opendut_types::peer::PeerDescriptor;

use crate::clusters::configurator::components::LeaderSelector;
use crate::clusters::configurator::types::UserClusterConfiguration;

#[component]
pub fn LeaderTab(
    cluster_configuration: RwSignal<UserClusterConfiguration>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {

    view! {
        <div>
            <LeaderSelector cluster_configuration peers />
        </div>
    }
}
