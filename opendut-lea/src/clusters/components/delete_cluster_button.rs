use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon};
use opendut_model::cluster::ClusterId;
use crate::app::use_app_globals;
use crate::clusters::overview::IsDeployed;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn DeleteClusterButton(
    cluster_id: Signal<ClusterId>,
    #[prop(into)] deployed_signal: Signal<IsDeployed>
) -> impl IntoView {

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
    let on_confirm = move || {
        let use_navigate = use_navigate.clone();
        let mut carl = globals.client.clone();
        let id = cluster_id.get();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let _ = carl.cluster.delete_cluster_descriptor(id).await; // TODO: Check the result and display a toast on failure.
            navigate_to(WellKnownRoutes::ClustersOverview, use_navigate);

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