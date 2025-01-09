use leptos::prelude::*;

use crate::clusters::configurator::DeviceSelector;
use crate::clusters::configurator::types::UserClusterConfiguration;

#[component]
pub fn DevicesTab(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {

    view! {
        <div>
            <DeviceSelector cluster_configuration=cluster_configuration/>
        </div>
    }
}
