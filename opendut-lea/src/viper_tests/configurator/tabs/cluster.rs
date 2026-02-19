use leptos::prelude::*;
use crate::viper_tests::configurator::components::ClusterSelector;
use crate::viper_tests::configurator::types::UserViperTestConfiguration;

#[component]
pub fn ClusterTab(viper_test_configuration: RwSignal<UserViperTestConfiguration>) -> impl IntoView {

    view! {
        <div>
            <ClusterSelector viper_test_configuration />
        </div>
    }
}
