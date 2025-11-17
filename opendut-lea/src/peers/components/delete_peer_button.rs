use leptos::{component, view, IntoView};
use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::peer::PeerId;
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

    let button_state = Signal::derive(move || {
        if  used_clusters_length.get() > 0 {
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

    view! {
        <Tooltip
            text="Peer can not be deleted while it is configured in a cluster."
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
