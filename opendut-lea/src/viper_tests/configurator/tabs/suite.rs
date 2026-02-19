use leptos::prelude::*;
use crate::viper_tests::configurator::components::TestSuiteSelector;
use crate::viper_tests::configurator::types::UserTestConfiguration;

#[component]
pub fn SuiteTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    view! {
        <TestSuiteSelector test_configuration />
    }
}
