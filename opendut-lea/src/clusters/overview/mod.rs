mod row;

use leptos::prelude::*;
use tracing::{debug, error};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;
use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor, ClusterId};

use crate::app::use_app_globals;
use crate::clusters::components::CreateClusterButton;
use crate::clusters::IsDeployed;
use crate::clusters::overview::row::Row;
use crate::components::{use_toaster, BasePageContainer, Breadcrumb, LoadingSpinner, Toast};

#[component]
pub fn ClustersOverview() -> impl IntoView {

    let globals = use_app_globals();
    let carl = globals.client;

    let refetch_cluster_descriptors = RwSignal::new(());

    let clusters = {
        let carl = carl.clone();

        LocalResource::new(move || {
            refetch_cluster_descriptors.track();
            let mut carl = carl.clone();
            async move {
                let mut clusters = carl.cluster.list_cluster_descriptors().await
                    .expect("Failed to request the list of clusters");

                clusters.sort_by(|cluster_a, cluster_b|
                    cluster_a.name.value().to_lowercase()
                        .cmp(&cluster_b.name.value().to_lowercase())
                );

                clusters
            }
        })
    };

    let refetch_cluster_deployments = RwSignal::new(());

    let cluster_deployments = {
        let carl = carl.clone();

        LocalResource::new(move || {
            refetch_cluster_deployments.track();

            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_deployments().await
                    .expect("Failed to request the list of cluster deployments")
            }
        })
    };


    let on_deploy = {
        let carl = carl.clone();
        let toaster = use_toaster();

        move |cluster_id: ClusterId| {
            let carl = carl.clone();
            let toaster = toaster.clone();

            move || {
                let mut carl = carl.clone();
                let toaster = toaster.clone();

                leptos::task::spawn_local(async move {
                    match carl.cluster.store_cluster_deployment(ClusterDeployment { id: cluster_id }).await {
                        Ok(cluster_id) => {
                            debug!("Successfully stored cluster deployment: {}", cluster_id);
                            toaster.toast(
                                Toast::builder()
                                    .simple("Successfully stored cluster deployment!")
                                    .success()
                            );
                        }
                        Err(cause) => {
                            error!("Failed to store cluster deployment <{}>, due to error: {:?}", cluster_id, cause);
                            match cause {
                                ClientError::UsageError(StoreClusterDeploymentError::IllegalPeerState { invalid_peers, .. }) => {
                                    toaster.toast(
                                        Toast::builder()
                                            .simple(format!("Failed to store cluster deployment! Peers already in use: {}", invalid_peers.iter().map(|peer| peer.to_string()).collect::<Vec<_>>().join(", ")))
                                            .error()
                                    );
                                }
                                _ => {
                                    toaster.toast(
                                        Toast::builder()
                                            .simple("Failed to store cluster deployment!")
                                            .error()
                                    );
                                }
                            };
                        }
                    }
                    refetch_cluster_deployments.notify();
                })
            }
        }
    };

    let on_undeploy = {
        let carl = carl.clone();
        let toaster = use_toaster();

        move |id: ClusterId| {
            let carl = carl.clone();
            let toaster = toaster.clone();

            move || {
                let mut carl = carl.clone();
                let toaster = toaster.clone();

                leptos::task::spawn_local(async move {
                    match carl.cluster.delete_cluster_deployment(id).await {
                        Ok(_) => {
                            toaster.toast(Toast::builder()
                                .simple("Successfully deleted cluster deployment!")
                                .success()
                            );
                        }
                        Err(_) => {
                            toaster.toast(Toast::builder()
                                .simple("Failed to delete cluster deployment!")
                                .error()
                            );
                        }
                    }
                    refetch_cluster_deployments.notify();
                })
            }
        }
    };

    let on_delete = move || {
        refetch_cluster_descriptors.notify();
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Clusters", "/clusters")
    ];

    view! {
        <BasePageContainer
            title="Clusters"
            breadcrumbs=breadcrumbs
            controls=view! {
                <CreateClusterButton />
            }
        >
            <table class="table is-hoverable is-fullwidth">
                <thead>
                    <tr>
                        <th class="is-narrow">"Deploy"</th>
                        <th class="is-narrow">"Health"</th>
                        <th>"Name"</th>
                        <th class="is-narrow has-text-centered">"Action"</th>
                    </tr>
                </thead>
                <tbody>
                    <Suspense
                        fallback=LoadingSpinner
                    >
                    {move || {
                        let on_deploy = on_deploy.clone();
                        let on_undeploy = on_undeploy.clone();

                        Suspend::new(async move {
                            let clusters = clusters.await;
                            let deployed_clusters = cluster_deployments.await
                                .iter()
                                .map(|cluster_deployment| cluster_deployment.id)
                                .collect::<Vec<_>>();

                            view! {
                                <For
                                    each = move || clusters.clone()
                                    key = |cluster| cluster.id
                                    children = { move |cluster_descriptor: ClusterDescriptor| {
                                        let cluster_id = cluster_descriptor.id;
                                        view! {
                                            <Row
                                                cluster_descriptor=RwSignal::new(cluster_descriptor)
                                                on_deploy=on_deploy(cluster_id)
                                                on_undeploy=on_undeploy(cluster_id)
                                                is_deployed = RwSignal::new(IsDeployed(deployed_clusters.contains(&cluster_id)))
                                                on_delete
                                            />
                                        }
                                    }}
                                />
                            }
                        })
                    }}
                    </Suspense>
                </tbody>
            </table>
        </BasePageContainer>
    }
}
