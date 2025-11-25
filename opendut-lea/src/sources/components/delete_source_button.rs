use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_model::viper::ViperSourceId;
use crate::app::use_app_globals;

#[component]
pub fn DeleteSourceButton<F>(
    source_id: Signal<ViperSourceId>,
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
        let id = source_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.viper.delete_viper_source_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted source: {:?}", source_id);
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted source successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to delete source <{:?}>, due to error: {cause:?}", source_id);
                    toaster.toast(
                        Toast::builder()
                            .simple("Failed to delete source!")
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
            label="Remove Source?"
            on_confirm
        />
    }
}
