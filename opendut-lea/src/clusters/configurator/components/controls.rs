use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::cluster::state::ClusterState;

use crate::app::use_app_globals;
use crate::clusters::components::{ClusterHealth, DeleteClusterButton, DeployToggle};
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::clusters::IsDeployed;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast, use_toaster};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls<OnDeploymentChanged>(
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    deployed_signal: Signal<IsDeployed>,
    cluster_state: Signal<ClusterState>,
    on_deployment_changed: OnDeploymentChanged,
) -> impl IntoView
where
    OnDeploymentChanged: Fn() + Clone + Send + 'static,
{

    let cluster_id = Signal::derive(move || {
        cluster_descriptor.get().id
    });
    let is_new_cluster = Signal::derive(move || {
        cluster_descriptor.get().is_new
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
            navigate_to(WellKnownRoutes::ClustersOverview, use_navigate.clone());
        }
    };

    view! {
        <div class="is-flex is-align-items-center">
            <div class="is-flex is-align-items-center">
                <DeployToggle
                    cluster_id
                    is_new_cluster
                    is_deployed=deployed_signal
                    on_deployment_changed
                />
            </div>
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
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    deployed_signal: Signal<IsDeployed>
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let set_is_new = create_write_slice(
        cluster_descriptor,
        |config, is_new| {
            config.is_new = is_new;
        },
    );

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
                            set_is_new.set(false);
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

    let tooltip_content = Box::new(move || {
        view! {
            Cluster can not be updated while it is deployed.
        }.into_any()
    });

    view! {
        <Tooltip
            text=tooltip_content
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
