use leptos::prelude::*;
use opendut_model::peer::PeerDescriptor;

use crate::clusters::configurator::components::LeaderSelector;
use crate::clusters::configurator::types::UserClusterDescriptor;

#[component]
pub fn LeaderTab(
    user_cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
    is_disabled: Signal<bool>,
) -> impl IntoView {

    view! {
        <div>
            <LeaderSelector user_cluster_descriptor peers is_disabled />
        </div>
    }
}
