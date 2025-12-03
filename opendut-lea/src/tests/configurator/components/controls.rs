use std::sync::Arc;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use opendut_model::viper::ViperRunDescriptor;
use crate::app::use_app_globals;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::tests::components::DeleteTestButton;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn Controls(
    configuration: RwSignal<UserTestConfiguration>,
    is_valid_test_configuration: Signal<bool>,
) -> impl IntoView {

    let test_id = Signal::derive(move || {
        configuration.get().id
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
        navigate_to(WellKnownRoutes::TestsOverview, use_navigate.clone())
    }};

    view! {
        <div class="is-flex">
            <SaveTestButton
                configuration
                is_valid_test_configuration
            />
            <div class="px-1" />
            <DeleteTestButton
                test_id
                button_color=ButtonColor::Danger
                on_delete
            />
        </div>
    }
}

#[component]
fn SaveTestButton(
    configuration: RwSignal<UserTestConfiguration>,
    is_valid_test_configuration: Signal<bool>,
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
        } else if is_valid_test_configuration.get() {
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

            let run_descriptor = ViperRunDescriptor::try_from(configuration.get_untracked());
            match run_descriptor {
                Ok(run_descriptor) => {
                    let test_id = run_descriptor.id;
                    let result = carl.viper.store_viper_run_descriptor(run_descriptor).await;
                    match result {
                        Ok(_) => {
                            debug!("Successfully stored test: {test_id}");
                            toaster.toast(
                                Toast::builder()
                                    .simple("Successfully stored test configuration.")
                                    .success(),
                            );
                            setter.set(false);
                        }
                        Err(cause) => {
                            error!("Failed to create test <{test_id}>, due to error: {cause:?}");
                            toaster.toast(Toast::builder().simple("Failed to store test!").error());
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to dispatch create test action, due to misconfiguration!\n  {error}");
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
            label="Save Test"
            on_action
        />
    }
}
