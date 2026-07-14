mod peer_selector;

use leptos::prelude::*;
use crate::viper_tests::configurator::tabs::peer::peer_selector::PeerSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn PeerTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <PeerSelector viper_test_run_descriptor />
    }
}
