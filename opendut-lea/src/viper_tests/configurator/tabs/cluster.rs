use leptos::prelude::*;
use crate::viper_tests::configurator::components::ClusterSelector;
use crate::viper_tests::configurator::types::UserTestConfiguration;

#[component]
pub fn ClusterTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    view! {
        <div>
            <ClusterSelector test_configuration />
        </div>
    }
}
