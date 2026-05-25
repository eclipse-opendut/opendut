use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_model::secret::SecretId;
use crate::app::use_app_globals;

#[component]
pub fn DeleteSecretButton<F>(
    #[prop(into)] secret_id: Signal<SecretId>,
    #[prop(into)] button_color: Signal<ButtonColor>,
    on_delete: F
) -> impl IntoView
where F: Fn() + Clone + Send + 'static {

    let globals = use_app_globals();
    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if pending.get() {
            ButtonState::Loading
        } else {
            ButtonState::Enabled
        }
    });

    let toaster = use_toaster();

    let on_confirm = move || {
        let on_delete = on_delete.clone();
        let mut carl = globals.client.clone();
        let id = secret_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.secret.delete_secret_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted secret: {:?}", secret_id);
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted secret successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to delete secret <{:?}>, due to error: {cause:?}", secret_id);
                    toaster.toast(
                        Toast::builder()
                            .simple("Failed to delete secret!")
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
            color=button_color
            size=ButtonSize::Normal
            state=button_state
            label="Remove Secret?"
            on_confirm
        />
    }
}
