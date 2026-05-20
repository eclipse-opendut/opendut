use leptos::prelude::*;
use crate::viper_tests::configurator::tabs::source::source_selector::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod source_selector;

#[component]
pub fn SourceTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <ViperTestSourceSelector viper_test_run_descriptor />
    }
}
