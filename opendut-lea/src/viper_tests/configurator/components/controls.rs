use std::sync::Arc;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use opendut_model::viper::ViperTestRunDescriptor;
use crate::app::use_app_globals;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_tests::components::DeleteViperTestButton;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn Controls(
    configuration: RwSignal<UserViperTestRunDescriptor>,
    #[prop(into)] is_valid_configuration: Signal<bool>,
) -> impl IntoView {

    let viper_test_id = Signal::derive(move || {
        configuration.get().id
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
        navigate_to(WellKnownRoutes::ViperTestsOverview, use_navigate.clone())
    }};

    view! {
        <div class="is-flex">
            <SaveViperTestButton
                configuration
                is_valid_configuration
            />
            <div class="px-1" />
            <DeleteViperTestButton
                viper_test_id
                button_color=ButtonColor::Danger
                on_delete
            />
        </div>
    }
}

#[component]
fn SaveViperTestButton(
    configuration: RwSignal<UserViperTestRunDescriptor>,
    is_valid_configuration: Signal<bool>,
) -> impl IntoView {

    let globals = use_app_globals();
    let toaster = use_toaster();

    let setter = create_write_slice(
        configuration,
        |config, input| {
            config.is_new = input;
        },
    );

    let pending = RwSignal::new(false);

    let button_state = Signal::derive(move || {
        if pending.get() {
            ButtonState::Loading
        } else if is_valid_configuration.get() {
            ButtonState::Enabled
        } else {
            ButtonState::Disabled
        }
    });

    let on_action = move || {
        let toaster = Arc::clone(&toaster);
        let mut carl = globals.client.clone();

        leptos::task::spawn_local(async move {
            pending.set(true);

            let viper_test_run_descriptor = ViperTestRunDescriptor::try_from(configuration.get_untracked());
            match viper_test_run_descriptor {
                Ok(viper_test_run_descriptor) => {
                    let viper_test_id = viper_test_run_descriptor.id;
                    let result = carl.viper.store_viper_test_run_descriptor(viper_test_run_descriptor).await;
                    match result {
                        Ok(_) => {
                            debug!("Successfully stored viper test: {viper_test_id}");
                            toaster.toast(
                                Toast::builder()
                                    .simple("Successfully stored viper test configuration.")
                                    .success(),
                            );
                            setter.set(false);
                        }
                        Err(cause) => {
                            error!("Failed to create viper test <{viper_test_id}>, due to error: {cause:?}");
                            toaster.toast(Toast::builder().simple("Failed to store viper test!").error());
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to dispatch create viper test action, due to misconfiguration!\n  {error}");
                }
            };

            pending.set(false);
        })
    };

    view! {
        <IconButton
            icon=FontAwesomeIcon::Save
            color=ButtonColor::Info
            size=ButtonSize::Normal
            state=button_state
            label="Save Viper Test"
            on_action
        />
    }
}
