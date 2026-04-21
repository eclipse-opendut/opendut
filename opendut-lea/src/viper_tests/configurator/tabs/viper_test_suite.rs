use leptos::prelude::*;
use crate::viper_tests::configurator::components::ViperTestParametersInput;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn ParametersTab(viper_test_configuration: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <ViperTestParametersInput viper_test_configuration />
    }
}
