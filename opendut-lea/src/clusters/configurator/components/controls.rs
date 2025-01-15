use std::sync::Arc;

use leptos::prelude::*;
use tracing::{debug, error};

use opendut_types::cluster::{ClusterConfiguration, ClusterId};

use crate::app::use_app_globals;
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::clusters::overview::IsDeployed;
use crate::components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, IconButton, Toast, use_toaster};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(cluster_configuration: ReadSignal<UserClusterConfiguration>, deployed_signal: RwSignal<IsDeployed>) -> impl IntoView {

    let (info_text, _) =
        signal({
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
                <SaveClusterButton cluster_configuration=cluster_configuration deployed_signal/>
                <DeleteClusterButton cluster_configuration=cluster_configuration deployed_signal/>
            </div>
        </div>
    }
}

#[component]
fn SaveClusterButton(cluster_configuration: ReadSignal<UserClusterConfiguration>, deployed_signal: RwSignal<IsDeployed>) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let store_action = Action::new(move |_| {
        let toaster = Arc::clone(&toaster);
        let configuration = ClusterConfiguration::try_from(cluster_configuration.get_untracked());
        let mut carl = globals.client.clone();
        async move {
            match configuration {
                Ok(configuration) => {
                    let result = carl.cluster.store_cluster_configuration(configuration).await;
                    match result {
                        Ok(cluster_id) => {
                            debug!("Successfully stored cluster configuration: {}", cluster_id);
                            toaster.toast(Toast::builder()
                                .simple("Successfully stored cluster configuration.")
                                .success()
                            );
                        }
                        Err(cause) => {
                            error!("Failed to store cluster <{}>, due to error: {:?}", "id", cause);
                            toaster.toast(Toast::builder()
                                .simple("Failed to store cluster configuration!")
                                .error()
                            );
                        }
                    }
                }
                Err(_) => {
                    error!("Failed to dispatch store cluster action, due to misconfiguration!");
                }
            }
        }
    });

    let button_state = Signal::derive(move || {
        if deployed_signal.get().0 {
            ButtonState::Disabled
        } else if store_action.pending().get() {
            ButtonState::Loading
        }
        else {
            cluster_configuration.with(|configuration| {
                if configuration.is_valid() {
                    ButtonState::Enabled
                }
                else {
                    ButtonState::Disabled
                }
            })
        }
    });

    view! {
        <IconButton
            icon=FontAwesomeIcon::Save
            color=ButtonColor::Info
            size=ButtonSize::Normal
            state=button_state
            label="Save Cluster"
            on_action=move || {
                store_action.dispatch(());
            }
        />
    }
}

#[component]
fn DeleteClusterButton(cluster_configuration: ReadSignal<UserClusterConfiguration>, deployed_signal: RwSignal<IsDeployed>) -> impl IntoView {

    let globals = use_app_globals();

    let delete_action = Action::new(move |id: &ClusterId| { //TODO Action doesn't work anymore, due to it not accept something that's not Send+Sync, which the gRPC stuff isn't; a Suspend seems to compile, but is it correct? Maybe use spawn_local instead?
        let id = id.to_owned();
        let mut carl = globals.client.clone();
        async move {
            let _ = carl.cluster.delete_cluster_configuration(id).await; // TODO: Check the result and display a toast on failure.
            navigate_to(WellKnownRoutes::ClustersOverview);
        }
    });

    let button_state = Signal::derive(move || {
        if deployed_signal.get().0 {
            ButtonState::Disabled
        } else if delete_action.pending().get() {
            ButtonState::Loading
        } else {
            ButtonState::Enabled
        }
    });
    
    view! {
        <ConfirmationButton
            icon=FontAwesomeIcon::TrashCan
            color=ButtonColor::Danger
            size=ButtonSize::Normal
            state=button_state
            label="Remove Cluster?"
            on_conform=move || {
                cluster_configuration.with_untracked(|config| {
                    delete_action.dispatch(config.id);
                });
            }
        />
    }
}
