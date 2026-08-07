use leptos::prelude::*;

use crate::clusters::configurator::components::ClusterNameInput;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::components::ReadOnlyInput;

#[component]
pub fn GeneralTab(user_cluster_descriptor: RwSignal<UserClusterDescriptor>) -> impl IntoView {

    let cluster_id = Signal::derive(move || user_cluster_descriptor.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="ID"
                value=cluster_id
            />
            <ClusterNameInput
                user_cluster_descriptor
            />
        </div>
    }
}
