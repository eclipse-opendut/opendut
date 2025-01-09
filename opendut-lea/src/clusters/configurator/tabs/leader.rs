use leptos::prelude::*;

use crate::clusters::configurator::components::LeaderSelector;
use crate::clusters::configurator::types::UserClusterConfiguration;

#[component]
pub fn LeaderTab(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {

    view! {
        <div>
            <LeaderSelector cluster_configuration=cluster_configuration/>
        </div>
    }
}