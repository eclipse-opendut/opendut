use std::sync::Arc;
use leptos::prelude::*;
use tracing::{debug, error};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use opendut_lea_components::tooltip::{Tooltip, TooltipDirection};
use opendut_model::viper::{ViperTestId, ViperTestName, ViperTestRunDescriptor};
use crate::app::use_app_globals;

#[component]
pub fn DuplicateViperTestButton(
    viper_test_run_descriptor: RwSignal<ViperTestRunDescriptor>,
    refetch_viper_tests: RwSignal<()>,
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let on_action = move || {
        let toaster = Arc::clone(&toaster);
        let mut carl = globals.client.clone();

        leptos::task::spawn_local(async move {
            let test_run_descriptor = viper_test_run_descriptor.get_untracked();

            let new_test_name = match ViperTestName::try_from(format!("{}-Copy", test_run_descriptor.name.value())) {
                Ok(name) => name,
                Err(error) => {
                    toaster.toast(
                        Toast::builder()
                            .simple(error.to_string())
                            .error(),
                    );
                    return;
                }
            };

            let viper_test_id = ViperTestId::random();
            let duplication = ViperTestRunDescriptor {
                id: viper_test_id,
                name: new_test_name,
                source: test_run_descriptor.source,
                peer: test_run_descriptor.peer,
                parameters: test_run_descriptor.parameters,
            };

            let result = carl.viper.store_viper_test_run_descriptor(duplication).await;
            match result {
                Ok(_) => {
                    refetch_viper_tests.notify();

                    debug!("Successfully stored viper test: {viper_test_id}");
                    toaster.toast(
                        Toast::builder()
                            .simple("Successfully stored viper test configuration.")
                            .success(),
                    );
                }
                Err(cause) => {
                    error!("Failed to create viper test <{viper_test_id}>, due to error: {cause:?}");
                    toaster.toast(Toast::builder().simple("Failed to store viper test!").error());
                }
            }
        })
    };

    view! {
        <Tooltip text="Duplicate Test Run" direction=TooltipDirection::Right>
            <IconButton
                icon=FontAwesomeIcon::Duplicate
                color=ButtonColor::White
                size=ButtonSize::Normal
                state=ButtonState::Enabled
                label="Duplicate VIPER Test"
                on_action
            />
        </Tooltip>
    }
}
