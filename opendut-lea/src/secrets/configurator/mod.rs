mod types;
mod components;
mod tabs;

use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::secret::SecretId;
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::secrets::configurator::components::Controls;
use crate::secrets::configurator::tabs::{GeneralTab, TabIdentifier};
use crate::secrets::configurator::types::UserSecretConfiguration;

#[component(transparent)]
pub fn SecretConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let (secret_configuration, secret_configuration_resource, is_valid_configuration) = {
        let secret_id = {
            let secret_id = params.with_untracked(|params| {
                params.get("id").and_then(|id| SecretId::try_from(id.as_str()).ok())
            });
            match secret_id {
                None => {
                    let use_navigate = use_navigate();
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Invalid SecretId"),
                        text: String::from("Could not parse the provided value as SecretId!"),
                        details: None,
                    }, use_navigate);
                    SecretId::random()
                }
                Some(secret_id) => {
                    secret_id
                }
            }
        };

        let secret_configuration = RwSignal::new(
            UserSecretConfiguration {
                id: secret_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid secret name.")),
                value: UserInputValue::Left(UserInputError::from("Enter a secret value (token).")),
                is_new: true,
            }
        );

        let secret_configuration_resource = LocalResource::new(move || {
            let mut carl = globals.client.clone();
            async move {
                let secrets = carl.secret.list_secret_descriptors().await
                    .unwrap_or_default();
                if let Some(configuration) = secrets.into_iter().find(|s| s.id == secret_id) {
                    secret_configuration.update(|user_configuration| {
                        user_configuration.name = UserInputValue::Right(configuration.name.value().to_owned());
                        user_configuration.value = match configuration.value {
                            opendut_model::secret::SecretValue::Token(token) => UserInputValue::Right(token),
                        };
                        user_configuration.is_new = false;
                    })
                }
            }
        });

        let is_valid_configuration = Memo::new(move |_| {
            secret_configuration.with(|config| {
                config.name.is_right()
                && config.value.is_right()
            })
        });

        (secret_configuration, secret_configuration_resource, is_valid_configuration)
    };

    let secret_id_string = create_read_slice(secret_configuration, |config| config.id.to_string());

    let breadcrumbs = Signal::derive(move || {
        let secret_id = secret_id_string.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Secrets", "/secrets"),
            Breadcrumb::new(&secret_id, format!("{secret_id}/configure")),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = secret_configuration.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {
        vec![
            Tab::from_title_and_href(
                String::from("General"),
                TabIdentifier::General.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !secret_configuration.read().is_valid())),
        ]
    });

    let active_tab = use_active_tab::<TabIdentifier>();

    view! {
        <BasePageContainer
            title="Configure Secret"
            subtitle=subtitle
            breadcrumbs=breadcrumbs
            controls=view! { <Controls configuration=secret_configuration is_valid_configuration /> }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                {
                    move || Suspend::new(async move {
                        secret_configuration_resource.await;

                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                { move || match active_tab.get() {
                                    TabIdentifier::General => view! { <GeneralTab secret_configuration /> }
                                }}
                            </Tabs>
                        }
                    })
                }
            </Suspense>
        </BasePageContainer>
    }
}
