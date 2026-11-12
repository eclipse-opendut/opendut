mod row;

use leptos::prelude::*;
use tracing::error;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, ErrorMessage, ErrorText, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor};

use crate::app::use_app_globals;
use crate::clusters::components::CreateClusterButton;
use crate::clusters::IsDeployed;
use crate::clusters::overview::row::Row;
use crate::components::{BasePageContainer, Breadcrumb};

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
                    .map_err(|error| {
                        error!("Failed to request the list of clusters: {error}");

                        ErrorText {
                            headline: String::from("Unable to load cluster descriptors"),
                            description: String::from("The cluster descriptors could not be loaded. Please check your connection and try again."),
                        }
                    })?;

                clusters.sort_by_key(|cluster| cluster.name.value().to_lowercase());

                Ok::<_, ErrorText>(clusters)
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
                    .map_err(|error| {
                        error!("Failed to request the list of cluster deployments: {error}");

                        ErrorText {
                            headline: String::from("Unable to load cluster deployments"),
                            description: String::from("The cluster deployments could not be loaded. Please check your connection and try again."),
                        }
                    })
            }
        })
    };

    let breadcrumbs = vec![
        Breadcrumb::new("Dashboard", "/"),
        Breadcrumb::new("Clusters", "/clusters")
    ];

    let table_headings = vec![
        TableHeading::new(String::from("Deploy")).set_narrow(),
        TableHeading::new(String::from("Health")).set_narrow(),
        TableHeading::new(String::from("Name")),
        TableHeading::new(String::from("Action")).set_narrow(),
    ];

    view! {
        <BasePageContainer
            title="Clusters"
            breadcrumbs
            controls=view! {
                <div class="buttons">
                    <CreateClusterButton />
                    <IconButton
                        icon=FontAwesomeIcon::ArrowsRotate
                        color=ButtonColor::Light
                        size=ButtonSize::Normal
                        state=ButtonState::Enabled
                        label="Refresh table of clusters"
                        on_action=move || {
                            refetch_cluster_descriptors.notify();
                        }
                    />
                </div>
            }
        >
            <OverviewTable headings=table_headings>
                { move || {
                    Suspend::new(async move {
                        let clusters = clusters.await;
                        let deployments = cluster_deployments.await;

                        match (clusters, deployments) {
                            (Ok(clusters), Ok(deployments)) => {
                                view! {
                                    <ClusterOverviewTable
                                        clusters
                                        deployments
                                        refetch_cluster_descriptors
                                    />
                                }.into_any()
                            }
                            (Err(error), _) | (_, Err(error)) => {
                                view! {
                                    <tr>
                                        <td colspan="999">
                                            <ErrorMessage
                                                text=error
                                            />
                                        </td>
                                    </tr>
                                }.into_any()
                            }
                        }
                    })
                }}
            </OverviewTable>
        </BasePageContainer>
    }
}

#[component]
fn ClusterOverviewTable(
    clusters: Vec<ClusterDescriptor>,
    deployments: Vec<ClusterDeployment>,
    refetch_cluster_descriptors: RwSignal<()>,
) -> impl IntoView {
    let deployed_clusters = deployments.iter()
        .map(|cluster_deployment| cluster_deployment.id)
        .collect::<Vec<_>>();

    let on_change = move || {
        refetch_cluster_descriptors.notify();
    };

    if clusters.is_empty() {
        view! {
            <tr>
                <td colspan="999" class="has-text-centered">"No clusters have been created yet"</td>
            </tr>
        }.into_any()
    } else {
        view! {
            <For
                each = move || clusters.clone()
                key = |cluster| cluster.id
                children = { move |cluster_descriptor: ClusterDescriptor| {
                    let cluster_id = cluster_descriptor.id;
                    view! {
                        <Row
                            cluster_descriptor=RwSignal::new(cluster_descriptor)
                            is_deployed = RwSignal::new(IsDeployed(deployed_clusters.contains(&cluster_id)))
                            on_change
                        />
                    }
                }}
            />
        }.into_any()
    }
}
