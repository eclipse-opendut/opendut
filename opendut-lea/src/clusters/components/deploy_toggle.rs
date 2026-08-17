use std::sync::Arc;

use leptos::prelude::*;
use tracing::{debug, error};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;
use opendut_lea_components::tooltip::Tooltip;
use opendut_lea_components::{Toggle, ToggleState};
use opendut_model::cluster::{ClusterDeployment, ClusterId};

use crate::app::use_app_globals;
use crate::clusters::IsDeployed;
use crate::components::{Toast, use_toaster};

#[component]
pub fn DeployToggle<OnDeploymentChanged>(
    #[prop(into)] cluster_id: Signal<ClusterId>,
    #[prop(into)] is_deployed: Signal<IsDeployed>,
    on_deployment_changed: OnDeploymentChanged,
) -> impl IntoView
where
    OnDeploymentChanged: Fn() + Clone + Send + 'static,
{
    let globals = use_app_globals();
    let toaster = use_toaster();

    let on_deploy = {
        let carl = globals.client.clone();
        let toaster = Arc::clone(&toaster);
        let on_deployment_changed = on_deployment_changed.clone();

        move || {
            let mut carl = carl.clone();
            let toaster = Arc::clone(&toaster);
            let cluster_id = cluster_id.get_untracked();
            let on_deployment_changed = on_deployment_changed.clone();

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
                    Err(source) => {
                        error!("Failed to store cluster deployment <{}>, due to error: {:?}", cluster_id, source);
                        match source {
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
                on_deployment_changed();
            })
        }
    };

    let on_undeploy = {
        let carl = globals.client.clone();
        let toaster = Arc::clone(&toaster);
        let on_deployment_changed = on_deployment_changed.clone();

        move || {
            let mut carl = carl.clone();
            let toaster = Arc::clone(&toaster);
            let cluster_id = cluster_id.get_untracked();
            let on_deployment_changed = on_deployment_changed.clone();

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
                on_deployment_changed();
            })
        }
    };

    let tooltip_text = Signal::derive(move || {
        if is_deployed.get().0 {
            "Deployment requested".to_string()
        } else {
            "Undeployed".to_string()
        }
    });

    view! {
        <Tooltip text=tooltip_text>
            <Toggle
                is_active=Signal::derive(move || is_deployed.get().0)
                state=ToggleState::Enabled
                on_action=move || {
                    if is_deployed.get().0 { on_undeploy() } else { on_deploy() }
                }
            />
        </Tooltip>
    }
}
