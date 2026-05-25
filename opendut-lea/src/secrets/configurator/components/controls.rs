use std::sync::Arc;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use tracing::{debug, error};
use opendut_lea_components::{use_toaster, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Toast};
use opendut_model::secret::SecretDescriptor;
use crate::app::use_app_globals;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::secrets::components::DeleteSecretButton;
use crate::secrets::configurator::types::UserSecretConfiguration;

#[component]
pub fn Controls(
    configuration: RwSignal<UserSecretConfiguration>,
    #[prop(into)] is_valid_configuration: Signal<bool>,
) -> impl IntoView {

    let secret_id = Signal::derive(move || {
        configuration.get().id
    });

    let use_navigate = use_navigate();
    let on_delete = { move || {
        navigate_to(WellKnownRoutes::SecretsOverview, use_navigate.clone())
    }};

    view! {
        <div class="is-flex">
            <SaveSecretButton
                configuration
                is_valid_configuration
            />
            <div class="px-1" />
            <DeleteSecretButton
                secret_id
                button_color=ButtonColor::Danger
                on_delete
            />
        </div>
    }
}

#[component]
fn SaveSecretButton(
    configuration: RwSignal<UserSecretConfiguration>,
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

            let secret_descriptor = SecretDescriptor::try_from(configuration.get_untracked());
            match secret_descriptor {
                Ok(secret_descriptor) => {
                    let secret_id = secret_descriptor.id;
                    let result = carl.secret.store_secret_descriptor(secret_descriptor).await;
                    match result {
                        Ok(_) => {
                            debug!("Successfully stored secret: {secret_id}");
                            toaster.toast(
                                Toast::builder()
                                    .simple("Successfully stored secret configuration.")
                                    .success(),
                            );
                            setter.set(false);
                        }
                        Err(cause) => {
                            error!("Failed to create secret <{secret_id}>, due to error: {cause:?}");
                            toaster.toast(Toast::builder().simple("Failed to store secret!").error());
                        }
                    }
                }
                Err(error) => {
                    error!("Failed to dispatch create secret action, due to misconfiguration!\n  {error}");
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
            label="Save Secret"
            on_action
        />
    }
}
