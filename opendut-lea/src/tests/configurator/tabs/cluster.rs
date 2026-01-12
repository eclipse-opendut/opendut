use leptos::prelude::*;
use crate::tests::configurator::components::ClusterSelector;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn ClusterTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    view! {
        <div>
            <ClusterSelector test_configuration />
        </div>
    }
}
