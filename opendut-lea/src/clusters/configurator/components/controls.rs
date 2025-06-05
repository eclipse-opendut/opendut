use std::sync::Arc;

use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};

use opendut_types::cluster::ClusterDescriptor;

use crate::app::use_app_globals;
use crate::clusters::configurator::types::UserClusterDescriptor;
use crate::clusters::overview::IsDeployed;
use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, IconButton, Toast, use_toaster};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(cluster_descriptor: ReadSignal<UserClusterDescriptor>, deployed_signal: Signal<IsDeployed>) -> impl IntoView {

    let info_text =
        Signal::derive(move || {
            if deployed_signal.get().0 {
                String::from("Cluster can not be updated or deleted while it is deployed.")
            } else {
                String::new()
            }
        });

    view! {
        <div class="is-flex is-align-items-center">
            <p style="color: #C11B17; margin-right: 8px" >{info_text}</p>
            <div class="buttons">
                <SaveClusterButton cluster_descriptor=cluster_descriptor deployed_signal/>
                <DeleteClusterButton cluster_descriptor=cluster_descriptor deployed_signal/>
            </div>
        </div>
    }
}

#[component]
fn SaveClusterButton(cluster_descriptor: ReadSignal<UserClusterDescriptor>, deployed_signal: Signal<IsDeployed>) -> impl IntoView {

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

    view! {
        <IconButton
            icon=FontAwesomeIcon::Save
            color=ButtonColor::Info
            size=ButtonSize::Normal
            state=button_state
            label="Save Cluster"
            on_action
        />
    }
}

#[component]
fn DeleteClusterButton(cluster_descriptor: ReadSignal<UserClusterDescriptor>, deployed_signal: Signal<IsDeployed>) -> impl IntoView {

    let globals = use_app_globals();

    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if deployed_signal.get().0 {
            ButtonState::Disabled
        } else if pending.get() {
            ButtonState::Loading
        } else {
            ButtonState::Enabled
        }
    });

    let use_navigate = use_navigate();
    let on_conform = move || {
        let use_navigate = use_navigate.clone();

        cluster_descriptor.with_untracked(|config| {
            let id = config.id.to_owned();
            let mut carl = globals.client.clone();

            leptos::task::spawn_local(async move {
                pending.set(true);

                let _ = carl.cluster.delete_cluster_descriptor(id).await; // TODO: Check the result and display a toast on failure.
                navigate_to(WellKnownRoutes::ClustersOverview, use_navigate);

                pending.set(false);
            });
        });
    };

    view! {
        <ConfirmationButton
            icon=FontAwesomeIcon::TrashCan
            color=ButtonColor::Danger
            size=ButtonSize::Normal
            state=button_state
            label="Remove Cluster?"
            on_conform
        />
    }
}
