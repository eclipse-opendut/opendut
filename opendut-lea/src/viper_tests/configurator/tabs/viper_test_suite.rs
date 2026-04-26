use leptos::prelude::*;
use crate::viper_tests::configurator::components::ViperTestSuiteSelector;
use crate::viper_tests::configurator::types::UserViperTestConfiguration;

#[component]
pub fn SuiteTab(viper_test_configuration: RwSignal<UserViperTestConfiguration>) -> impl IntoView {

    view! {
        <ViperTestSuiteSelector viper_test_configuration />
    }
}
