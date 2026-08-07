use leptos::prelude::*;
use crate::viper_tests::configurator::tabs::source::source_selector::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod source_selector;

#[component]
pub fn SourceTab(user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <ViperTestSourceSelector user_test_run_descriptor />
    }
}
