use leptos::prelude::*;

use opendut_lea_components::{ErrorMessage, ErrorText};
use opendut_lea_components::InfoMessage;
use opendut_model::peer::PeerDescriptor;

use crate::clusters::configurator::DeviceSelector;
use crate::clusters::configurator::types::UserClusterDescriptor;

#[component]
pub fn DevicesTab(
    user_cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Result<Vec<PeerDescriptor>, ErrorText>>,
    is_disabled: Signal<bool>,
) -> impl IntoView {

    view! {
        <div>
            <InfoMessage text="The Devices of individual Peers to connect into the Cluster." />
            { move || match peers.get() {
                Ok(peers) => {
                    let peers = RwSignal::new(peers).read_only();
                    view! {
                        <DeviceSelector user_cluster_descriptor peers is_disabled />
                    }.into_any()
                }
                Err(error)=> {
                    view! {
                        <ErrorMessage
                            text=error
                        />
                    }.into_any()
                }
            }}
        </div>
    }
}
