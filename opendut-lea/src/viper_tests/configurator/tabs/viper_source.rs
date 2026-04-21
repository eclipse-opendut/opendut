use leptos::prelude::*;
use crate::viper_tests::configurator::components::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn SourceTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <ViperTestSourceSelector viper_test_run_descriptor />
    }
}
