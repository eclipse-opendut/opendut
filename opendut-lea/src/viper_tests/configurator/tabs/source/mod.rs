use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::viper_tests::configurator::tabs::source::source_selector::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod source_selector;

#[component]
pub fn SourceTab(user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <InfoMessage
            text="Select the Source to load your test code from."
        />
        <ViperTestSourceSelector user_test_run_descriptor />
    }
}
