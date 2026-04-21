use leptos::prelude::*;
use crate::viper_tests::configurator::components::ClusterSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn ClusterTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <div>
            <ClusterSelector viper_test_run_descriptor />
        </div>
    }
}
