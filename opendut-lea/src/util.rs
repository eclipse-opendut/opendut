use std::collections::HashSet;
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::topology::DeviceId;

pub mod view {
    use leptos::prelude::*;

    pub fn join_with_comma_spans<T: RenderHtml + 'static>(elements: Vec<View<T>>) -> Vec<AnyView> {
        let elements_length = elements.len();

        let mut elements_with_separator = Vec::new();

        for (index, element) in elements.into_iter().enumerate() {
            elements_with_separator.push(element.into_any());

            if index < (elements_length - 1) {
                elements_with_separator.push(
                    view! { <span>", "</span> }.into_any()
                );
            }
        }
        elements_with_separator
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
