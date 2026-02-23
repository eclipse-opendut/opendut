use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_lea_components::Toggle;
use opendut_model::cluster::ClusterDeployment;
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::cluster::state::ClusterState;

use crate::app::use_app_globals;
use crate::clusters::components::DeleteClusterButton;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::clusters::IsDeployed;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast, use_toaster};
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::clusters::components::ClusterHealth;

#[component]
pub fn Controls(
    cluster_descriptor: ReadSignal<UserClusterDescriptor>,
    deployed_signal: Signal<IsDeployed>,
    cluster_state: Signal<ClusterState>,
    refetch_cluster_deployments: RwSignal<()>
) -> impl IntoView {

    let cluster_id = Signal::derive(move || {
        cluster_descriptor.get().id
    });
    
    let globals = use_app_globals();
    let toaster = use_toaster();

    let on_deploy = {
        let carl = globals.client.clone();
        let toaster = Arc::clone(&toaster);

        move || {
            let mut carl = carl.clone();
            let toaster = Arc::clone(&toaster);
            let cluster_id = cluster_id.get_untracked();

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
    };

    let on_undeploy = {
        let carl = globals.client.clone();
        let toaster = Arc::clone(&toaster);

        move || {
            let mut carl = carl.clone();
            let toaster = Arc::clone(&toaster);
            let cluster_id = cluster_id.get_untracked();

            leptos::task::spawn_local(async move {
                match carl.cluster.delete_cluster_deployment(cluster_id).await {
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
    };

    let deploy_tooltip_text = Signal::derive(move || {
        if deployed_signal.get().0 {
            "Deployment requested".to_string()
        } else {
            "Undeployed".to_string()
        }
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
            navigate_to(WellKnownRoutes::ClustersOverview, use_navigate.clone());
        }
    };

    view! {
        <div class="is-flex is-align-items-center">
            <Tooltip text=deploy_tooltip_text>     
                <Toggle
                    is_active=Signal::derive(move || deployed_signal.get().0)
                    on_action=move || {
                        if deployed_signal.get().0 { on_undeploy() } else { on_deploy() }
                    }
                />
            </Tooltip>
            <div class="px-2" />
            <ClusterHealth state=cluster_state />
            <div class="px-2" />
            <SaveClusterButton
                cluster_descriptor
                deployed_signal
            />
            <div class="px-1" />
            <DeleteClusterButton
                cluster_id
                deployed_signal
                button_color=ButtonColor::Danger
                on_delete
            />
        </div>
    }
}

#[component]
fn SaveClusterButton(
    cluster_descriptor: ReadSignal<UserClusterDescriptor>,
    deployed_signal: Signal<IsDeployed>
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if deployed_signal.get().0 {
            ButtonState::Disabled
        } else if pending.get() {
            ButtonState::Loading
        }
        else {
            cluster_descriptor.with(|configuration| {
                if configuration.is_valid() {
                    ButtonState::Enabled
                }
                else {
                    ButtonState::Disabled
                }
            })
        }
    });

    let on_action = move || {
        let toaster = Arc::clone(&toaster);
        let configuration = ClusterDescriptor::try_from(cluster_descriptor.get_untracked());
        let mut carl = globals.client.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            match configuration {
                Ok(configuration) => {
                    let result = carl.cluster.store_cluster_descriptor(configuration).await;
                    match result {
                        Ok(cluster_id) => {
                            debug!("Successfully stored cluster descriptor: {}", cluster_id);
                            toaster.toast(Toast::builder()
                                .simple("Successfully stored cluster descriptor.")
                                .success()
                            );
                        }
                        Err(cause) => {
                            error!("Failed to store cluster <{}>, due to error: {:?}", "id", cause);
                            toaster.toast(Toast::builder()
                                .simple("Failed to store cluster descriptor!")
                                .error()
                            );
                        }
                    }
                }
                Err(_) => {
                    error!("Failed to dispatch store cluster action, due to misconfiguration!");
                }
            }

            pending.set(false);
        })
    };

    let hide_tooltip = Signal::derive(move || {
        !deployed_signal.get().0
    });

    view! {
        <Tooltip
            text="Cluster can not be updated while it is deployed."
            direction=TooltipDirection::Right
            is_hidden=hide_tooltip
        >
            <IconButton
                icon=FontAwesomeIcon::Save
                color=ButtonColor::Info
                size=ButtonSize::Normal
                state=button_state
                label="Save Cluster"
                on_action
            />
        </Tooltip>
    }
}
