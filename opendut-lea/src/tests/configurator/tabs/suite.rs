use leptos::prelude::*;
use crate::tests::configurator::components::TestSuiteSelector;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn SuiteTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    view! {
        <TestSuiteSelector test_configuration />
    }
}
