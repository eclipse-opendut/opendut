use leptos::*;
use leptos::html::Div;
use leptos_use::on_click_outside;
use serde::{Deserialize, Serialize};

use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};

use crate::app::{ExpectGlobals, use_app_globals};
use crate::clusters::components::CreateClusterButton;
use crate::components::{BasePageContainer, Breadcrumb, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, health, IconButton, Initialized, Toast, use_toaster};
use crate::components::health::Health;

#[component]
pub fn ClustersOverview() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

        let globals = use_app_globals();

        let clusters = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.cluster.list_cluster_configurations().await
                    .expect("Failed to request the list of clusters")
            }
        });

        let cluster_deployments = create_local_resource(|| {}, move |_| {
            let mut carl = globals.expect_client();
            async move {
                carl.cluster.list_cluster_deployments().await
                    .expect("Failed to request the list of cluster deployments")
            }
        });

        let deploy_cluster = create_action(move |id: &ClusterId| {
            let toaster = use_toaster();
            let mut carl = globals.expect_client();
            let id = Clone::clone(id);
            async move {
                match carl.cluster.store_cluster_deployment(ClusterDeployment { id }).await {
                    Ok(_) => {
                        toaster.toast(Toast::builder()
                            .simple("Successfully stored cluster deployment!")
                            .success()
                        );
                    }
                    Err(_) => {
                        toaster.toast(Toast::builder()
                            .simple("Failed to store cluster deployment!")
                            .error()
                        );
                    }
                }
                cluster_deployments.refetch();
            }
        });

        let undeploy_cluster = create_action(move |id: &ClusterId| {
            let toaster = use_toaster();
            let mut carl = globals.expect_client();
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
                                cluster_configuration=create_rw_signal(cluster_configuration)
                                on_deploy=move || deploy_cluster.dispatch(cluster_id)
                                on_undeploy=move || undeploy_cluster.dispatch(cluster_id)
                                is_deployed = create_rw_signal(IsDeployed(deployed_clusters().contains(&cluster_id)))
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
pub struct IsDeployed(bool);

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

    let dropdown_active = create_rw_signal(false);
    let dropdown = create_node_ref::<Div>();

    let _ = on_click_outside(dropdown, move |_| dropdown_active.set(false) );

    let (health_state, _) = create_signal({
        if is_deployed.get().0 {
            health::State {
                kind: health::StateKind::Green,
                text: String::from("Connected. No errors."),
            }
        }
        else {
            health::State {
                kind: health::StateKind::Unknown,
                text: String::from("Disconnected"),
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
