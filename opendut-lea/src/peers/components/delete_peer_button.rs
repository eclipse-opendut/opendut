use leptos::{component, view, IntoView};
use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonStateSignalProvider, ConfirmationButton, DoorhangerButton, FontAwesomeIcon, Toast};
use opendut_model::peer::PeerId;
use crate::app::use_app_globals;
use crate::routing;

#[component]
pub fn DeletePeerButton<F>(
    peer_id: Signal<PeerId>,
    #[prop(into)] used_clusters_length: Signal<usize>,
    on_delete: F
) -> impl IntoView
where F: Fn() + Clone + Send + 'static {

    let globals = use_app_globals();

    let pending = RwSignal::new(false);

    let button_state = Signal::from(pending).derive_loading();

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

    let delete_button = move || {
        let on_confirm = on_confirm.clone();

        if used_clusters_length.get() > 0 {
            view! {
                <DoorhangerButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Danger
                    size=ButtonSize::Normal
                    state=button_state
                    label="Remove Peer?"
                >
                    <div style="white-space: nowrap">
                        "Peer can not be removed while it is configured in "{used_clusters_length}
                        <a class="has-text-link" href=routing::path::clusters_overview>" cluster(s)"</a>
                    </div>
                </DoorhangerButton>
            }.into_any()
        } else {
            view! {
                <ConfirmationButton
                    icon=FontAwesomeIcon::TrashCan
                    color=ButtonColor::Danger
                    size=ButtonSize::Normal
                    state=button_state
                    label="Remove Peer?"
                    on_confirm
                />
            }.into_any()
        }
    };

    view! {
        <div>
            { delete_button }
        </div>
    }
}
