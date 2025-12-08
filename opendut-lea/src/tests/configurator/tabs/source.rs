use leptos::prelude::*;
use crate::tests::configurator::components::TestSourceSelector;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn SourceTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    view! {
        <TestSourceSelector test_configuration />
    }
}
