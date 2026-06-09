use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_model::viper::ViperSourceId;
use crate::app::use_app_globals;

#[component]
pub fn DeleteViperSourceButton<F>(
    #[prop(into)] viper_source_id: Signal<ViperSourceId>,
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
        let id = viper_source_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.viper.delete_viper_source_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted viper source: {}", viper_source_id.get_untracked());
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted viper source successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to delete viper source <{}>, due to error: {cause}", viper_source_id.get_untracked());
                    toaster.toast(
                        Toast::builder()
                            .simple(cause.to_string())
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
            label="Remove Viper Source?"
            on_confirm
        />
    }
}
