use leptos::prelude::*;
use tracing::{error, info};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use opendut_model::viper::{ViperRunDeployment, ViperRunId, ViperTestId};
use crate::app::use_app_globals;

#[component]
pub fn DeployViperTestButton(
    #[prop(into)] test_id: Signal<ViperTestId>,
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let on_click = move || {
        let mut carl = globals.client.clone();
        let toaster = toaster.clone();
        let deployment = ViperRunDeployment {
            run_id: ViperRunId::random(),
            test_id: test_id.get(),
        };

        leptos::task::spawn_local(async move {
            let result = carl.viper.store_viper_run_deployment(deployment).await;

            match result {
                Ok(_) => {
                    info!("Successfully stored VIPER test run deployment: {:?}", test_id);
                    toaster.toast(
                        Toast::builder()
                            .simple("Deployed VIPER test run successfully.")
                            .success()
                    );
                }
                Err(cause) => {
                    error!("Failed to deploy VIPER test run for test <{:?}>, due to error: {cause:?}", test_id);
                    toaster.toast(
                        Toast::builder()
                            .simple("Failed to deploy VIPER test run!")
                            .error()
                    )
                }
            }
        });
    };

    view! {
        <IconButton
            icon=FontAwesomeIcon::Play
            color=ButtonColor::Success
            size=ButtonSize::Small
            state=ButtonState::Enabled
            label="Deploy VIPER Test"
            on_action=on_click
        />
    }
}
