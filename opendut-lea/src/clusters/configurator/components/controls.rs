use leptos::*;
use tracing::{error, info};

use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use crate::app::{ExpectGlobals, use_app_globals};

use crate::components::{ButtonColor, ButtonSize, ButtonState, ButtonStateSignalProvider, ConfirmationButton, FontAwesomeIcon, IconButton};
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn Controls(cluster_configuration: ReadSignal<UserClusterConfiguration>) -> impl IntoView {

    view! {
        <div class="buttons">
            <SaveClusterButton cluster_configuration=cluster_configuration />
            <DeleteClusterButton cluster_configuration=cluster_configuration />
        </div>
    }
}

#[component]
fn SaveClusterButton(cluster_configuration: ReadSignal<UserClusterConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let store_action = create_action(move |_: &()| {

        let configuration = ClusterConfiguration::try_from(cluster_configuration.get_untracked());

        async move {
            // TODO: Implement some kind of toast to show the result of the store action.
            match configuration {
                Ok(configuration) => {
                    let mut carl = globals.expect_client();
                    let result = carl.cluster.store_cluster_configuration(configuration).await;
                    match result {
                        Ok(cluster_id) => {
                            info!("Successfully stored cluster: {}", cluster_id);
                        }
                        Err(cause) => {
                            error!("Failed to store cluster <{}>, due to error: {:?}", "id", cause);
                        }
                    }
                }
                Err(_) => {
                    error!("Failed to dispatch store cluster action, due to misconfiguration!");
                }
            }
        }
    });

    let button_state = MaybeSignal::derive(move || {
        if store_action.pending().get() {
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
fn DeleteClusterButton(cluster_configuration: ReadSignal<UserClusterConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let delete_action = create_action(move |id: &ClusterId| {
        let id = id.to_owned();
        async move {
            let mut carl = globals.expect_client();
            let _ = carl.cluster.delete_cluster_configuration(id).await; // TODO: Check the result and display a toast on failure.
            navigate_to(WellKnownRoutes::ClustersOverview);
        }
    });

    let button_state = delete_action
        .pending()
        .derive_loading();

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
