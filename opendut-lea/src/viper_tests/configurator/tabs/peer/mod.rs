mod peer_selector;

use leptos::prelude::*;
use opendut_lea_components::{ErrorMessage, ErrorMessageType, ErrorText, InfoMessage};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use crate::viper_tests::configurator::tabs::peer::peer_selector::PeerSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn PeerTab(
    user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    peers: ReadSignal<Result<Vec<PeerDescriptor>, ErrorText>>,
    clusters: ReadSignal<Result<Vec<ClusterDescriptor>, ErrorText>>,
    #[prop(into)] on_retry_peers: Callback<()>,
    #[prop(into)] on_retry_clusters: Callback<()>,
) -> impl IntoView {

    view! {
        <InfoMessage
            text="Select the Peer to run your VIPER Test on."
        />
        { move || if let Err(error) = clusters.get() {
            view! {
                <ErrorMessage
                    text=error
                    error_type=ErrorMessageType::Warning
                    on_retry=on_retry_clusters
                />
            }.into_any()
        } else { ().into_any() }}

        { move || match peers.get() {
            Ok(peers) => {
                let peers = RwSignal::new(peers).read_only();
                view! {
                    <PeerSelector user_test_run_descriptor peers clusters />
                }.into_any()
            }
            Err(error)=> {
                view! {
                    <ErrorMessage
                        text=error
                        on_retry=on_retry_peers
                    />
                }.into_any()
            }
        }}
    }
}
