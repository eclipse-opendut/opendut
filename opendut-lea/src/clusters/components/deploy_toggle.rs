use crate::api::clusters::list_cluster_peer_states;
use std::sync::Arc;

use leptos::prelude::*;
use tracing::{debug, error};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_lea_components::{ErrorIndicator, ErrorText, Toggle, ToggleState};
use opendut_model::cluster::{ClusterDeployment, ClusterId};
use opendut_model::peer::PeerId;
use opendut_model::peer::state::PeerMemberState;
use crate::app::use_app_globals;
use crate::clusters::IsDeployed;
use crate::components::{Toast, use_toaster};

#[component]
pub fn DeployToggle<OnDeploymentChange>(
    #[prop(into)] cluster_id: Signal<ClusterId>,
    #[prop(into, default=Signal::from(false))] is_new_cluster: Signal<bool>,
    #[prop(into)] is_deployed: Signal<Option<IsDeployed>>,
    #[prop(into, default=Signal::from(()))] refetch_peer_states: Signal<()>,
    on_deployment_change: OnDeploymentChange,
    #[prop(optional)] tooltip_direction: TooltipDirection,
) -> impl IntoView
where
    OnDeploymentChange: Fn() + Clone + Send + 'static,
{
    let globals = use_app_globals();
    let toaster = use_toaster();
    let pending = RwSignal::new(false);

    let on_deploy = {
        let carl = globals.client.clone();
        let toaster = Arc::clone(&toaster);
        let on_deployment_changed = on_deployment_change.clone();

        move || {
            let mut carl = carl.clone();
            let toaster = Arc::clone(&toaster);
            let cluster_id = cluster_id.get_untracked();
            let on_deployment_changed = on_deployment_changed.clone();

            leptos::task::spawn_local(async move {
                pending.set(true);

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
                pending.set(false);
            })
        }
    };

    let on_undeploy = {
        let carl = globals.client.clone();
        let toaster = Arc::clone(&toaster);
        let on_deployment_changed = on_deployment_change.clone();

        move || {
            let mut carl = carl.clone();
            let toaster = Arc::clone(&toaster);
            let cluster_id = cluster_id.get_untracked();
            let on_deployment_changed = on_deployment_changed.clone();

            leptos::task::spawn_local(async move {
                pending.set(true);

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
                pending.set(false);
            })
        }
    };

    let cluster_peer_states = list_cluster_peer_states(
        Signal::derive(move || {
            if is_new_cluster.get() { None } else { Some(cluster_id.get()) }
        }),
        Some(Signal::derive(move || {
            is_deployed.track();
            refetch_peer_states.track();
        })),
    );

    let deployment_blocker = Signal::derive(move || {
        match cluster_peer_states.get() {
            Some(Ok(Some(peer_states))) => {
                let invalid_peers = peer_states.iter()
                    .filter(|(_, state)| matches!(&state.member, PeerMemberState::Blocked { .. }))
                    .map(|(peer_id, _)| *peer_id)
                    .collect::<Vec<_>>();
                if invalid_peers.is_empty() {
                    None
                } else {
                    Some(DeploymentBlocker::Peers(invalid_peers))
                }
            }
            Some(Err(error)) => Some(DeploymentBlocker::Message(error.description)),
            _ => None,
        }
    });

    let is_active = Signal::derive(move || matches!(is_deployed.get(), Some(IsDeployed(true))));
    let toggle_state = Signal::derive(move || {
        if pending.get() {
            ToggleState::Loading
        } else if is_new_cluster.get() || is_deployed.get().is_none() {
            ToggleState::Disabled
        } else if cluster_peer_states.get().is_none() {
            ToggleState::Loading
        } else if deployment_blocker.get().is_some() && !is_active.get() {
            ToggleState::Disabled
        } else {
            ToggleState::Enabled
        }
    });

    let tooltip_content = Arc::new(move || {
        view! {
            <DeploymentTooltipContent
                is_new_cluster
                is_deployed=is_active
                blocked_deployment=deployment_blocker
            />
        }.into_any()
    });

    view! {
        <Tooltip text=tooltip_content direction=tooltip_direction>
            <Toggle
                is_active
                state=toggle_state
                on_action=move || {
                    if is_active.get_untracked() { on_undeploy() } else { on_deploy() }
                }
            />
        </Tooltip>
        <Show when=move || is_deployed.get().is_none()>
            <ErrorIndicator
                text=ErrorText {
                    headline: String::from("Unable to load cluster deployments."),
                    description: String::new(),
                }
                direction=tooltip_direction
            />
        </Show>
    }
}

#[derive(Clone)]
enum DeploymentBlocker {
    Peers(Vec<PeerId>),
    Message(String),
}

#[component]
fn DeploymentTooltipContent(
    is_new_cluster: Signal<bool>,
    #[prop(into)] is_deployed: Signal<bool>,
    blocked_deployment: Signal<Option<DeploymentBlocker>>,
) -> impl IntoView {

    move || {
        if is_new_cluster.get() {
            return view! {
                "Save the cluster first."
            }.into_any();
        }

        if !is_deployed.get()
            && let Some(blocked_deployment) = blocked_deployment.get().as_ref() {
                return match blocked_deployment {
                    DeploymentBlocker::Peers(peers) => {
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

                    DeploymentBlocker::Message(message) => {
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
