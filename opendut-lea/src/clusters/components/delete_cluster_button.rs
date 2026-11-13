use std::sync::Arc;
use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::cluster::ClusterId;
use crate::app::use_app_globals;
use crate::clusters::IsDeployed;

#[component]
pub fn DeleteClusterButton<F>(
    cluster_id: Signal<ClusterId>,
    #[prop(into)] deployed_signal: Signal<Option<IsDeployed>>,
    #[prop(into)] button_color: Signal<ButtonColor>,
    on_delete: F,
) -> impl IntoView
where F: Fn() + Clone + Send + 'static {

    let globals = use_app_globals();
    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if !matches!(deployed_signal.get(), Some(IsDeployed(false)))  {
            ButtonState::Disabled
        } else if pending.get() {
            ButtonState::Loading
        } else {
            ButtonState::Enabled
        }
    });

    let toaster = use_toaster();

    let on_confirm = move || {
        let on_delete = on_delete.clone();
        let mut carl = globals.client.clone();
        let id = cluster_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.cluster.delete_cluster_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted cluster: {:?}", cluster_id);
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted cluster successfully.")
                            .success()
                    );
                },
                Err(error) => {
                    error!("Failed to delete cluster <{id}>: {error}");
                    toaster.toast(
                        Toast::builder()
                            .simple("Failed to delete cluster!")
                            .error()
                    );
                }
            }

            pending.set(false);
        });
    };

    let hide_tooltip = Signal::derive(move || {
        !matches!(button_state.get(), ButtonState::Disabled)
    });

    let tooltip_content = Arc::new(move || {
        view! {
            {move || match deployed_signal.get() {
                Some(IsDeployed(true)) => "Cluster can not be deleted while it is deployed.",
                Some(IsDeployed(false)) => "",
                None => "Cluster can not be deleted because its deployment status could not be determined.",
            }}
        }.into_any()
    });

    view! {
        <Tooltip
            text=tooltip_content
            direction=TooltipDirection::Right
            is_hidden=hide_tooltip
        >
            <ConfirmationButton
                icon=FontAwesomeIcon::TrashCan
                color=button_color
                size=ButtonSize::Normal
                state=button_state
                label="Remove Cluster?"
                on_confirm
            />
        </Tooltip>
    }
}
