use leptos::prelude::*;

use crate::clusters::configurator::components::ClusterNameInput;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::components::ReadOnlyInput;

#[component]
pub fn GeneralTab(cluster_descriptor: RwSignal<UserClusterDescriptor>) -> impl IntoView {

    let cluster_id = Signal::derive(move || cluster_descriptor.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Cluster ID"
                value=cluster_id
            />
            <ClusterNameInput
                cluster_descriptor=cluster_descriptor
            />
        </div>
    }
}
