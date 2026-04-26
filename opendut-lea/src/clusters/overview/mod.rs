mod row;

use leptos::prelude::*;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, OverviewTable, TableHeading};
use opendut_model::cluster::ClusterDescriptor;

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

    let on_delete = move || {
        refetch_cluster_descriptors.notify();
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
                                                is_deployed = RwSignal::new(IsDeployed(deployed_clusters.contains(&cluster_id)))
                                                on_deployment_changed=move || refetch_cluster_deployments.notify()
                                                on_delete
                                            />
                                        }
                                    }}
                                />
                            }
                        })
                    }}
            </OverviewTable>
        </BasePageContainer>
    }
}
