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
use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, health, IconButton, Initialized, Toast, use_toaster};
use crate::components::health::Health;

#[component]
pub fn ClustersOverview() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

        let globals = use_app_globals();

        let clusters = LocalResource::new(move || {
            let mut carl = globals.client;
            async move {
                carl.cluster.list_cluster_configurations().await
                    .expect("Failed to request the list of clusters")
            }
        });

        let cluster_deployments = LocalResource::new(move || {
            let mut carl = globals.client;
            async move {
                carl.cluster.list_cluster_deployments().await
                    .expect("Failed to request the list of cluster deployments")
            }
        });

        let deploy_cluster = Action::new(move |cluster_id: &ClusterId| {
            let toaster = use_toaster();
            let mut carl = globals.client;
            let cluster_id = Clone::clone(cluster_id);
            async move {
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
                cluster_deployments.refetch();
            }
        });

        let undeploy_cluster = Action::new(move |id: &ClusterId| {
            let toaster = use_toaster();
            let mut carl = globals.client;
            let id = Clone::clone(id);
            async move {
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
                cluster_deployments.refetch();
            }
        });

        let deployed_clusters = move || {
            match cluster_deployments.get() {
                Some(deployed_clusters) => {
                    deployed_clusters.iter().map(|cluster_deployment| cluster_deployment.id).collect::<Vec<_>>()
                }
                None => Vec::new()
            }
        };
        let rows = move || {
            match clusters.get() {
                Some(configurations) => {
                    configurations.iter().cloned().map(|cluster_configuration| {
                        let cluster_id = cluster_configuration.id;
                        view! {
                            <Row
                                cluster_configuration=RwSignal::new(cluster_configuration)
                                on_deploy=move || deploy_cluster.dispatch(cluster_id)
                                on_undeploy=move || undeploy_cluster.dispatch(cluster_id)
                                is_deployed = RwSignal::new(IsDeployed(deployed_clusters().contains(&cluster_id)))
                            />
                        }
                    }).collect::<Vec<_>>()
                }
                None => {
                    Vec::new()
                }
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
            </BasePageContainer>
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
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

    let (health_state, _) = signal({
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
