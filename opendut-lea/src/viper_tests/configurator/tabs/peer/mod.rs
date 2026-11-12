mod peer_selector;

use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::viper_tests::configurator::tabs::peer::peer_selector::PeerSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn PeerTab(user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <InfoMessage
            text="" // TODO: description (Why peer in VIPER test?)
        />
        <PeerSelector user_test_run_descriptor />
    }
}
