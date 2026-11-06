use std::sync::Arc;
use leptos::{component, view, IntoView};
use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::peer::PeerId;
use opendut_model::viper::{ViperTestId, ViperTestName};
use crate::app::use_app_globals;

#[component]
pub fn DeletePeerButton<F>(
    peer_id: Signal<PeerId>,
    #[prop(into)] used_clusters_length: Signal<usize>,
    #[prop(into)] button_color: Signal<ButtonColor>,
    on_delete: F
) -> impl IntoView
where F: Fn() + Clone + Send + 'static {

    let globals = use_app_globals();
    let pending = RwSignal::new(false);
    let toaster = use_toaster();

    let used_by_viper: LocalResource<Result<Vec<(ViperTestId, ViperTestName)>, String>> = {
        let carl = globals.client.clone();
        let toaster = toaster.clone();

        LocalResource::new(move || {
            let peer_id = peer_id.get();
            let mut carl = carl.clone();
            let toaster = toaster.clone();
            pending.set(true);

            async move {
                let result = carl.viper.list_viper_test_run_descriptors().await
                    .map(|tests| tests.into_iter()
                        .filter(|test| test.peer == peer_id)
                        .map(|test| (test.id, test.name))
                        .collect())
                    .map_err(|cause| {
                        error!("Failed to load peer <{peer_id}>, due to error: {cause}");
                        toaster.toast(Toast::builder()
                            .simple("Failed to check peer usage.")
                            .error());
                        cause.to_string()
                    });
                pending.set(false);
                result
            }
        })
    };

    let button_state = Signal::derive(move || {
        if used_clusters_length.get() > 0 {
            ButtonState::Disabled
        } else if pending.get() {
            ButtonState::Loading
        } else {
            match used_by_viper.get() {
                None => ButtonState::Loading,
                Some(Ok(tests)) if tests.is_empty() => ButtonState::Enabled,
                Some(_) => ButtonState::Disabled,
            }
        }
    });

    let on_confirm = move || {
        let on_delete = on_delete.clone();
        let mut carl = globals.client.clone();
        let id = peer_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.peers.delete_peer_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted peer: {:?}", peer_id);
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted peer successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to delete peer <{:?}>, due to error: {cause:?}", peer_id);
                    toaster.toast(
                        Toast::builder()
                            .simple("Failed to delete peer!")
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
            Peer can not be deleted while it is configured in a cluster.
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
                label="Remove Peer?"
                on_confirm
            />
        </Tooltip>
    }
}
