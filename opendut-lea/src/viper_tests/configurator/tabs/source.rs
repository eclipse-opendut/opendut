use leptos::prelude::*;
use crate::viper_tests::configurator::components::TestSourceSelector;
use crate::viper_tests::configurator::types::UserTestConfiguration;

#[component]
pub fn SourceTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    view! {
        <TestSourceSelector test_configuration />
    }
}
