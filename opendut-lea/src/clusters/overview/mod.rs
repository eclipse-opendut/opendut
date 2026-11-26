use crate::util::client_error_description;
mod row;

use leptos::prelude::*;
use tracing::error;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, ErrorMessage, InfoMessage, ErrorMessageTableRow, ErrorMessageType, ErrorText, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor};
use crate::api::clusters::list_cluster_descriptors;
use crate::app::use_app_globals;
use crate::clusters::components::CreateClusterButton;
use crate::clusters::IsDeployed;
use crate::clusters::overview::row::Row;
use crate::components::{BasePageContainer, Breadcrumb, LoadingSpinner};

#[component]
pub fn ClustersOverview() -> impl IntoView {

    let globals = use_app_globals();
    let carl = globals.client;

    let refetch_cluster_descriptors = RwSignal::new(());

    let clusters = list_cluster_descriptors();

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

                        let error_description = client_error_description(
                            &error,
                            "The cluster deployments could not be loaded. Please check your connection and try again.",
                        );

                        ErrorText {
                            headline: String::from("Unable to load cluster deployments"),
                            description: error_description,
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
                            refetch_cluster_deployments.notify();
                        }
                    />
                </div>
            }
        >
            <InfoMessage
                text="Clusters establish the network between Peers, allowing your connected ECUs to communicate like in one testbench."
            />
            <Transition fallback=|| ()>
                {move || match cluster_deployments.get() {
                    Some(Err(error)) => view! {
                        <ErrorMessage
                            text=error
                            on_retry=move || cluster_deployments.refetch()
                            error_type=ErrorMessageType::Warning
                        />
                    }.into_any(),
                    Some(Ok(_)) | None => ().into_any(),
                }}
            </Transition>
            <OverviewTable headings=table_headings>
                { move || {
                    Suspend::new(async move {
                        let clusters_result = clusters.await;

                        match clusters_result {
                            Ok(clusters) => {
                                view! {
                                    <ClusterOverviewTable
                                        clusters
                                        deployments=cluster_deployments
                                        refetch_cluster_deployments
                                        refetch_cluster_descriptors
                                    />
                                }.into_any()
                            }
                            Err(error)=> {
                                view! {
                                    <ErrorMessageTableRow
                                        text=error
                                        on_retry=move || clusters.refetch()
                                    />
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
    deployments: LocalResource<Result<Vec<ClusterDeployment>, ErrorText>>,
    refetch_cluster_deployments: RwSignal<()>,
    refetch_cluster_descriptors: RwSignal<()>,
) -> impl IntoView {

    let on_deployment_change = move || {
        refetch_cluster_deployments.notify();
    };

    let on_delete = move || {
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
                        <Transition fallback=move || view! {
                            <tr>
                                <td colspan="999"><LoadingSpinner /></td>
                            </tr>
                        }>
                            {move || {
                                let is_deployed = Memo::new(move |_| {
                                    match deployments.get() {
                                        Some(Ok(deployments)) => Some(IsDeployed(
                                            deployments.iter().any(|deployment| deployment.id == cluster_id)
                                        )),
                                        Some(Err(_)) | None => None,
                                    }
                                });

                                view! {
                                    <Row
                                        cluster_descriptor=RwSignal::new(cluster_descriptor.clone())
                                        is_deployed=is_deployed.into()
                                        refetch_cluster_deployments
                                        on_deployment_change
                                        on_delete
                                    />
                                }
                            }}
                        </Transition>
                    }
                }}
            />
        }.into_any()
    }
}
