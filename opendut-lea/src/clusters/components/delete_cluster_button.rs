use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_model::cluster::ClusterId;
use crate::app::use_app_globals;
use crate::clusters::IsDeployed;

#[component]
pub fn DeleteClusterButton<F>(
    cluster_id: Signal<ClusterId>,
    #[prop(into)] deployed_signal: Signal<IsDeployed>,
    on_delete: F,
) -> impl IntoView
where F: Fn() + Clone + Send + 'static {

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

    view! {
        <ConfirmationButton
            icon=FontAwesomeIcon::TrashCan
            color=ButtonColor::Danger
            size=ButtonSize::Normal
            state=button_state
            label="Remove Cluster?"
            on_confirm
        />
    }
}
