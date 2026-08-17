use std::sync::Arc;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use opendut_model::viper::ViperSourceDescriptor;
use crate::app::use_app_globals;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_sources::components::DeleteViperSourceButton;
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn Controls(
    configuration: RwSignal<UserViperSourceConfiguration>,
    #[prop(into)] is_valid_configuration: Signal<bool>,
) -> impl IntoView {

    let viper_source_id = Signal::derive(move || {
        configuration.get().id
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
        navigate_to(WellKnownRoutes::ViperSourcesOverview, use_navigate.clone())
    }};

    view! {
        <div class="is-flex">
            <SaveViperSourceButton
                configuration
                is_valid_configuration
            />
            <div class="px-1" />
            <DeleteViperSourceButton
                viper_source_id
                button_color=ButtonColor::Danger
                on_delete
            />
        </div>
    }
}

#[component]
fn SaveViperSourceButton(
    configuration: RwSignal<UserViperSourceConfiguration>,
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

            let viper_source_descriptor = ViperSourceDescriptor::try_from(configuration.get_untracked());
            match viper_source_descriptor {
                Ok(viper_source_descriptor) => {
                    let viper_source_id = viper_source_descriptor.id;
                    let result = carl.viper.store_viper_source_descriptor(viper_source_descriptor).await;
                    match result {
                        Ok(_) => {
                            debug!("Successfully stored viper source: {viper_source_id}");
                            toaster.toast(
                                Toast::builder()
                                    .simple("Successfully stored viper source configuration.")
                                    .success(),
                            );
                            setter.set(false);
                        }
                        Err(source) => {
                            error!("Failed to create viper source <{viper_source_id}>, due to error: {source:?}");
                            toaster.toast(Toast::builder().simple("Failed to store viper source!").error());
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to dispatch create viper source action, due to misconfiguration!\n  {error}");
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
            label="Save Viper Source"
            on_action
        />
    }
}
