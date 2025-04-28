use leptos::prelude::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use serde::{Deserialize, Serialize};
use tracing::{debug, error};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};

use crate::app::use_app_globals;
use crate::clusters::components::CreateClusterButton;
use crate::components::{health, use_toaster, BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, LoadingSpinner, Toast};
use crate::components::health::Health;

#[component]
pub fn ClustersOverview() -> impl IntoView {

    let globals = use_app_globals();
    let carl = globals.client;

    let clusters = {
        let carl = carl.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_configurations().await
                    .expect("Failed to request the list of clusters")
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

    let deployed_clusters = LocalResource::new(move || async move {
        cluster_deployments.await.iter()
            .map(|cluster_deployment| cluster_deployment.id)
            .collect::<Vec<_>>()
    });
    let rows = move || {
        let on_deploy = on_deploy.clone();
        let on_undeploy = on_undeploy.clone();

        async move {
            let mut clusters = clusters.await;
            clusters.sort_by(|cluster_a, cluster_b|
                cluster_a.name.value().cmp(cluster_b.name.value())
            );

            let deployed_clusters = deployed_clusters.await;

            clusters.iter().cloned().map(|cluster_configuration| {
                let cluster_id = cluster_configuration.id;
                view! {
                    <Row
                        cluster_configuration=RwSignal::new(cluster_configuration)
                        on_deploy=on_deploy(cluster_id)
                        on_undeploy=on_undeploy(cluster_id)
                        is_deployed = RwSignal::new(IsDeployed(deployed_clusters.contains(&cluster_id)))
                    />
                }
            }).collect::<Vec<_>>()
        }
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
            <Suspense
                fallback=LoadingSpinner
            >
            {move || {
                let rows = rows.clone();

                Suspend::new(async move {
                    let rows = rows().await;
                    view! {
                        <table class="table is-hoverable is-fullwidth">
                            <thead>
                                <tr>
                                    <th class="is-narrow">"Health"</th>
                                    <th>"Name"</th>
                                    <th class="is-narrow">"Action"</th>
                                </tr>
                            </thead>
                            <tbody>
                                { rows }
                            </tbody>
                        </table>
                    }
                })
            }}
            </Suspense>
        </BasePageContainer>
    }
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct IsDeployed(pub bool);

#[component]
fn Row<OnDeployFn, OnUndeployFn>(
    cluster_configuration: RwSignal<ClusterConfiguration>,
    on_deploy: OnDeployFn,
    on_undeploy: OnUndeployFn,
    is_deployed: RwSignal<IsDeployed>,
) -> impl IntoView
where
    OnDeployFn: Fn() + 'static,
    OnUndeployFn: Fn() + 'static,
{

    let cluster_id = create_read_slice(cluster_configuration,
        |cluster_configuration| {
            cluster_configuration.id
        }
    );

    let cluster_name = create_read_slice(cluster_configuration,
        |cluster_configuration| {
            Clone::clone(&cluster_configuration.name).to_string()
        }
    );

    let configurator_href = move || format!("/clusters/{}/configure/general", cluster_id.get());

    let dropdown_active = RwSignal::new(false);
    let dropdown = NodeRef::<Div>::new();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    let health_state = Signal::derive(move || {
        if is_deployed.get().0 {
            health::State {
                kind: health::StateKind::Yellow,
                text: String::from("Marked for deployment. Deployment-State unknown."),
            }
        }
        else {
            health::State {
                kind: health::StateKind::Unknown,
                text: String::from("Undeployed"),
            }
        }
    });

    view! {
        <tr>
            <td class="is-vcentered">
                <Health state=health_state />
            </td>
            <td class="is-vcentered">
                <a href={ configurator_href } >{ cluster_name }</a>
            </td>
            <td class="is-vcentered">
                <div class="is-pulled-right">
                    <div class="dropdown is-right" class=("is-active", move || dropdown_active.get())>
                        <div class="dropdown-trigger">
                            <IconButton
                                icon=FontAwesomeIcon::EllipsisVertical
                                color=ButtonColor::White
                                size=ButtonSize::Normal
                                state=ButtonState::Enabled
                                label="Show Cluster Action Menu"
                                on_action=move || {
                                    dropdown_active.update(|value| *value = !*value);
                                }
                            />
                        </div>
                        <div node_ref=dropdown class="dropdown-menu">
                            <div class="dropdown-content">
                                <button
                                    class="button is-white is-fullwidth is-justify-content-flex-start"
                                    aria-label="Deploy Cluster"
                                    on:click=move |_| {
                                        dropdown_active.set(false);
                                        on_deploy();
                                    }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-cloud-arrow-up"></i>
                                    </span>
                                    <span>"Deploy"</span>
                                </button>
                                <button
                                    class="button is-white is-fullwidth is-justify-content-flex-start"
                                    aria-label="Undeploy Cluster"
                                    on:click=move |_| {
                                        dropdown_active.set(false);
                                        on_undeploy();
                                    }
                                >
                                    <span class="icon">
                                        <i class="fa-solid fa-cloud-arrow-down"></i>
                                    </span>
                                    <span>"Undeploy"</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </td>
        </tr>
    }
}
