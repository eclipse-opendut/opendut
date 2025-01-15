use leptos::prelude::*;

use crate::clusters::configurator::components::ClusterNameInput;
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::components::ReadOnlyInput;

#[component]
pub fn GeneralTab(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {

    let cluster_id = Signal::derive(move || cluster_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Cluster ID"
                value=cluster_id
            />
            <ClusterNameInput
                cluster_configuration=cluster_configuration
            />
        </div>
    }
}
