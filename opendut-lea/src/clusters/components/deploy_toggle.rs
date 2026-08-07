use std::sync::Arc;

use leptos::prelude::*;
use tracing::{debug, error};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::{ListClusterPeerStatesResponse, StoreClusterDeploymentError};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_lea_components::{Toggle, ToggleState};
use opendut_model::cluster::{ClusterDeployment, ClusterId};
use opendut_model::peer::PeerId;
use opendut_model::peer::state::PeerMemberState;
use crate::app::use_app_globals;
use crate::clusters::IsDeployed;
use crate::components::{Toast, use_toaster};

#[component]
pub fn DeployToggle<OnDeploymentChanged>(
    #[prop(into)] cluster_id: Signal<ClusterId>,
    #[prop(into, default=Signal::from(false))] is_new_cluster: Signal<bool>,
    #[prop(into)] is_deployed: Signal<IsDeployed>,
    on_deployment_changed: OnDeploymentChanged,
    #[prop(optional)] tooltip_direction: TooltipDirection,
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

    let blocked_cluster_deployment = {
        let carl = globals.client.clone();
        let cluster_id = cluster_id.get_untracked();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            let is_new_cluster = is_new_cluster.get();
            let _ = is_deployed.get();

            async move {
                if is_new_cluster {
                    return None;
                }

                let states = carl.cluster.list_cluster_peer_states(cluster_id).await
                    .expect("Failed to request the list of cluster's peer state.");
                match states {
                    ListClusterPeerStatesResponse::Success { peer_states } => {
                        let invalid_peers = peer_states.iter()
                            .filter(|(_, peer_state)| {
                                matches!(&peer_state.member, PeerMemberState::Blocked { .. })
                            })
                            .map(|(peer_id, _)| peer_id.to_owned())
                            .collect::<Vec<_>>();

                        if invalid_peers.is_empty() {
                            None
                        } else {
                            Some(BlockedDeployment::Peers(invalid_peers))
                        }
                    }
                    ListClusterPeerStatesResponse::Failure { message } => {
                        Some(BlockedDeployment::Message(message))
                    }
                }
            }
        })
    };

    move || {
        let on_deploy = on_deploy.clone();
        let on_undeploy = on_undeploy.clone();

        Suspend::new(async move {
            let blocked_deployment = blocked_cluster_deployment.await;

            let is_deployed = Signal::derive(move || is_deployed.get().0);

            let show_error = Signal::derive({
                let blocked_deployment = blocked_deployment.clone();
                move || blocked_deployment.is_some() && !is_deployed.get()
            });

            let toggle_state = Signal::derive(move || {
                if show_error.get() || is_new_cluster.get() {
                    ToggleState::Disabled
                } else {
                    ToggleState::Enabled
                }
            });

            let tooltip_content = Box::new(move || {
                view! {
                    <DeploymentTooltipContent is_new_cluster is_deployed blocked_deployment />
                }.into_any()
            });

            view! {
                <Tooltip text=tooltip_content direction=tooltip_direction>
                    <Toggle
                        is_active=is_deployed
                        state=toggle_state
                        on_action=move || {
                            if is_deployed.get() { on_undeploy() } else { on_deploy() }
                        }
                    />
                </Tooltip>
            }
        })
    }
}

#[derive(Clone)]
enum BlockedDeployment {
    Peers(Vec<PeerId>),
    Message(String),
}

#[component]
fn DeploymentTooltipContent(
    is_new_cluster: Signal<bool>,
    is_deployed: Signal<bool>,
    blocked_deployment: Option<BlockedDeployment>,
) -> impl IntoView {

    move || {
        if is_new_cluster.get() {
            return view! {
                "Save the cluster first."
            }.into_any();
        }

        if !is_deployed.get()
            && let Some(blocked_deployment) = blocked_deployment.as_ref() {
                return match blocked_deployment {
                    BlockedDeployment::Peers(peers) => {
                        let amount = peers.len();

                        let message = if amount == 1 {
                            "1 Peer is already in use:".to_string()
                        } else {
                            format!("{amount} Peers are already in use:")
                        };

                        let peer_links = peers
                            .iter()
                            .enumerate()
                            .map(|(index, peer_id)| {
                                let href = format!("/peers/{peer_id}/configure/general");

                                view! {
                                    <span>
                                        {if index == 0 { "" } else { ", " }}
                                        <a href=href>{peer_id.to_string()}</a>
                                    </span>
                                }
                            })
                            .collect_view();

                        view! {
                            <span>{message} " " {peer_links}</span>
                        }
                            .into_any()
                    }

                    BlockedDeployment::Message(message) => {
                        view! { { message.to_owned() } }.into_any()
                    }
                };
            }


        if is_deployed.get() {
            view! { "Deployment requested" }.into_any()
        } else {
            view! { "Undeployed" }.into_any()
        }
    }
}
