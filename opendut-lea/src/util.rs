use std::collections::HashSet;
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::topology::DeviceId;

pub mod view {
    use leptos::prelude::*;
    use opendut_lea_components::FontAwesomeIcon;

    pub fn join_with_comma_spans<T: RenderHtml + 'static>(
        elements: Vec<View<T>>,
    ) -> AnyView {
        let len = elements.len();

        elements.into_iter()
            .enumerate()
            .map(|(index, element)| {
                view! {
                {element}
                {(index < len - 1).then(|| view! {
                    <span>", "</span>
                })}
            }
            })
            .collect::<Vec<_>>()
            .into_any()
    }

    pub fn tooltip_content_for_configurator_errors(configurator: &str) -> AnyView {
        view! {
            { configurator } " cannot be saved while configuration errors "
            <span class="icon has-text-danger">
                <i class=FontAwesomeIcon::CircleExclamation.as_class() />
            </span>
            " remain."
        }.into_any()
    }
}

pub fn list_configured_clusters_for_peer(devices_in_peer: Vec<DeviceId>, clusters: Vec<ClusterDescriptor>) -> Vec<ClusterDescriptor> {
    let mut clusters_without_duplicates = HashSet::new();

    let mut clusters = clusters
        .into_iter()
        .filter(|cluster| {
            cluster
                .devices
                .iter()
                .any(|device| devices_in_peer.contains(device))
        })
        .filter(|cluster| clusters_without_duplicates.insert(cluster.id))
        .collect::<Vec<_>>();

    clusters.sort_by(|a, b| a.name.value().cmp(b.name.value()));

    clusters
}
