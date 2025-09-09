use leptos::prelude::*;
use opendut_model::peer::PeerDescriptor;

use crate::clusters::configurator::components::LeaderSelector;
use crate::clusters::configurator::types::UserClusterDescriptor;

#[component]
pub fn LeaderTab(
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {

    view! {
        <div>
            <LeaderSelector cluster_descriptor peers />
        </div>
    }
}
