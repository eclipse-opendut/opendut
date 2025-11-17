use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::cluster::ClusterDescriptor;

use crate::app::use_app_globals;
use crate::clusters::components::DeleteClusterButton;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::clusters::IsDeployed;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast, use_toaster};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(
    cluster_descriptor: ReadSignal<UserClusterDescriptor>,
    deployed_signal: Signal<IsDeployed>
) -> impl IntoView {

    let cluster_id = Signal::derive(move || {
        cluster_descriptor.get().id
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
            navigate_to(WellKnownRoutes::ClustersOverview, use_navigate.clone());
        }
    };

    view! {
        <div class="is-flex">
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
