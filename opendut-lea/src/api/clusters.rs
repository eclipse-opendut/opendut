use leptos::prelude::{LocalResource, RwSignal, Track};
use tracing::error;
use opendut_lea_components::ErrorText;
use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor};
use crate::app::use_app_globals;
use crate::util::client_error_description;

pub fn list_cluster_descriptors() -> LocalResource<Result<Vec<ClusterDescriptor>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        let mut carl = carl.clone();
        async move {
            let clusters = carl.cluster.list_cluster_descriptors().await
                .map_err(|error| {
                    error!("Failed to request the list of clusters: {error}");

                    let error_description = client_error_description(
                        &error,
                        "The cluster descriptors could not be loaded. Please check your connection and try again.",
                    );

                    ErrorText {
                        headline: String::from("Unable to load cluster descriptors"),
                        description: error_description,
                    }
                })?;

            Ok(clusters)
        }
    })
}
pub fn list_cluster_deployments(
    refetch_signal: Option<RwSignal<()>>,
) -> LocalResource<Result<Vec<ClusterDeployment>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let deployed = carl.cluster.list_cluster_deployments().await
                .map_err(|error| {
                    error!("Failed to request the list of cluster deployments: {error}");

                    let description = client_error_description(
                        &error,
                        "The cluster deployments could not be loaded. Please check your connection and try again.",
                    );
                    ErrorText {
                        headline: String::from("Unable to load cluster deployments"),
                        description
                    }
                })?;

            Ok(deployed)
        }
    })
}
