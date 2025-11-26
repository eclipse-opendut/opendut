use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, ConfirmationButton, FontAwesomeIcon, Toast};
use opendut_model::viper::ViperRunId;
use crate::app::use_app_globals;

#[component]
pub fn DeleteTestButton<F>(
    test_id: Signal<ViperRunId>,
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
        let id = test_id.get();
        let toaster = toaster.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let result = carl.viper.delete_viper_run_descriptor(id).await;

            match result {
                Ok(_) => {
                    info!("Successfully deleted test: {:?}", test_id);
                    on_delete();
                    toaster.toast(
                        Toast::builder()
                            .simple("Deleted test successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to delete test <{:?}>, due to error: {cause:?}", test_id);
                    toaster.toast(
                        Toast::builder()
                            .simple("Failed to delete test!")
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
            label="Remove Test?"
            on_confirm
        />
    }
}
