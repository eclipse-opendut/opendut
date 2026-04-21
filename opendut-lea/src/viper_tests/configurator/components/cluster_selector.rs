use leptos::prelude::*;
use opendut_lea_components::{SelectionTable, SelectionTableRow};
use opendut_model::cluster::ClusterDescriptor;
use crate::app::use_app_globals;
use crate::viper_tests::configurator::types::{ClusterSelection, UserViperTestRunDescriptor};

#[component]
pub fn ClusterSelector(viper_test_configuration: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let globals = use_app_globals();

    let registered_clusters = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_descriptors().await
                    .expect("Failed to request the list of clusters")
            }
        })
    };

    let (getter, setter) = create_slice(viper_test_configuration,
        |config| {
            Clone::clone(&config.cluster)
        },
        |config, input| {
            config.cluster = input;
        }
    );

    let clusters = Signal::derive(move || {
        if let Some(mut clusters) = registered_clusters.get() {
            clusters
                .sort_by(|cluster_a, cluster_b| {
                    cluster_a.name.value().to_lowercase()
                        .cmp(&cluster_b.name.value().to_lowercase())
            });

            let rows = clusters.iter().map(|cluster_descriptor| {
                let ClusterDescriptor { id, name, .. } = cluster_descriptor;
                let id = id.to_owned();
                let name = name.value().to_owned();

                SelectionTableRow {
                    id: Clone::clone(&id),
                    cells: vec![name, id.to_string()]
                }
            }).collect::<Vec<_>>();

            if clusters.is_empty() {
                setter.set(ClusterSelection::Left(String::from("No clusters available.")));
            } else if matches!(getter.get(), ClusterSelection::Left(_)) {
                setter.set(ClusterSelection::Left(String::from("Select a cluster.")));
            }

            rows
        } else {
            Vec::new()
        }
    });

    let header = vec![
        String::new(),
        String::from("Name"),
        String::from("Cluster ID")
    ];

    view! {
        <SelectionTable
            header
            rows=clusters
            getter
            setter
        />
    }
}
