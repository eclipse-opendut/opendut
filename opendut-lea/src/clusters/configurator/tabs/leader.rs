use leptos::prelude::*;
use opendut_lea_components::{ErrorMessage, ErrorText};
use opendut_lea_components::InfoMessage;
use opendut_model::peer::PeerDescriptor;

use crate::clusters::configurator::components::LeaderSelector;
use crate::clusters::configurator::types::UserClusterDescriptor;

#[component]
pub fn LeaderTab(
    user_cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Result<Vec<PeerDescriptor>, ErrorText>>,
    is_disabled: Signal<bool>,
    #[prop(into)] on_retry: Callback<()>,
) -> impl IntoView {

    view! {
        <div>
            <InfoMessage
                text="The Leader is a Peer that takes on a more central role in the Cluster. \
                    It performs healthchecks and for clusters with three or more peers, the communication is routed through the Leader. \
                    As such, you should ideally pick a peer that is physically located in the middle, relative to the other peers."
            />
            { move || match peers.get() {
                Ok(peers) => {
                    let peers = RwSignal::new(peers).read_only();
                    view! {
                        <fieldset prop:disabled=move || is_disabled.get()>
                            <LeaderSelector user_cluster_descriptor peers is_disabled />
                        </fieldset>
                    }.into_any()
                }
                Err(error)=> {
                    view! {
                        <ErrorMessage
                            text=error
                            on_retry
                        />
                    }.into_any()
                }
            }}
        </div>
    }
}
