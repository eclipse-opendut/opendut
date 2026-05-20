mod cluster_selector;

use leptos::prelude::*;
use crate::viper_tests::configurator::tabs::cluster::cluster_selector::ClusterSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn ClusterTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <div>
            <ClusterSelector viper_test_run_descriptor />
        </div>
    }
}
