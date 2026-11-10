use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
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
            <InfoMessage
                text="The Leader is a Peer that takes on a more central role in the Cluster. \
                    It performs healthchecks and for clusters with three or more peers, the communication is routed through the Leader. \
                    As such, you should pick a peer that is physically located in the middle of the other peers, if possible."
            />
            <LeaderSelector user_cluster_descriptor peers is_disabled />
        </div>
    }
}
