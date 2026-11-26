use std::collections::HashMap;
use opendut_model::peer::{PeerId, state::PeerState};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::{GetClusterDescriptorError};
use opendut_model::cluster::ClusterId;
use leptos::prelude::{Get, LocalResource, Signal, Track};
use opendut_lea_components::ErrorText;
use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor};
use crate::app::use_app_globals;
use crate::api::client_error_text;

pub fn list_cluster_descriptors(
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Vec<ClusterDescriptor>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let mut clusters = carl.cluster.list_cluster_descriptors().await
                .map_err(|error| {
                    client_error_text(
                        &error,
                        "Unable to load cluster descriptors",
                        "The cluster descriptors could not be loaded. Please check your connection and try again.",
                    )
                })?;

            clusters.sort_by_key(|cluster| cluster.name.value().to_lowercase());

            Ok(clusters)
        }
    })
}
pub fn list_cluster_deployments(
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Vec<ClusterDeployment>, ErrorText>> {
    let globals = use_app_globals();
    let carl = globals.client;

    LocalResource::new(move || {
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            let deployed = carl.cluster.list_cluster_deployments().await
                .map_err(|error| {
                    client_error_text(
                        &error,
                        "Unable to load cluster deployments",
                        "The cluster deployments could not be loaded. Please check your connection and try again.",
                    )
                })?;

            Ok(deployed)
        }
    })
}

pub fn get_cluster_descriptor(
    id: impl Into<Signal<ClusterId>>,
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Option<ClusterDescriptor>, ErrorText>> {
    let id = id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let id = id.get();
        refetch_signal.track();

        let mut carl = carl.clone();
        async move {
            match carl.cluster.get_cluster_descriptor(id).await {
                Ok(descriptor) => Ok(Some(descriptor)),
                Err(ClientError::UsageError(GetClusterDescriptorError::ClusterNotFound { .. })) => Ok(None),
                Err(error) => Err(client_error_text(
                    &error,
                    "Unable to load cluster descriptor",
                    "The cluster descriptor could not be loaded. Please check your connection and try again.",
                )),
            }
        }
    })
}

pub fn list_cluster_peer_states(
    cluster_id: impl Into<Signal<Option<ClusterId>>>,
    refetch_signal: Option<Signal<()>>,
) -> LocalResource<Result<Option<HashMap<PeerId, PeerState>>, ErrorText>> {
    let cluster_id = cluster_id.into();
    let carl = use_app_globals().client;

    LocalResource::new(move || {
        let cluster_id = cluster_id.get();
        refetch_signal.track();
        let mut carl = carl.clone();

        async move {
            let Some(cluster_id) = cluster_id else {
                return Ok(None);
            };

            carl.cluster.list_cluster_peer_states(cluster_id).await
                .map(Some)
                .map_err(|error| {
                    client_error_text(
                        &error,
                        "Unable to load cluster peer states",
                        "The cluster peer states could not be loaded. Please check your connection and try again.",
                    )
                })
        }
    })
}
