use std::collections::HashMap;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, UserInputError, UserInputValue};
use opendut_lea_components::tabs::Tabs;
use opendut_model::viper::{ViperRunId, ViperRunParameterValue};
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::tests::configurator::components::Controls;
use crate::tests::configurator::tabs::{GeneralTab, ParameterTab, TabIdentifier};
use crate::tests::configurator::types::UserTestConfiguration;

mod tabs;
mod types;
mod components;

#[component(transparent)]
pub fn TestConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let (test_configuration, test_configuration_resource, is_valid_test_configuration) = {
        let test_id = {
            let test_id = params.with_untracked(|params| {
                params.get("id").and_then(|id| ViperRunId::try_from(id.as_str()).ok())
            });
            match test_id {
                None => {
                    let use_navigate = use_navigate();
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Invalid TestId"),
                        text: String::from("Could not parse the provided value as TestId!"),
                        details: None,
                    }, use_navigate);
                    ViperRunId::random()
                }
                Some(test_id) => {
                    test_id
                }
            }
        };

        let test_configuration = RwSignal::new(
            UserTestConfiguration {
                id: test_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid test name.")),
                source: UserInputValue::Left(String::from("Select a test source.")),
                suite: UserInputValue::Left(String::from("Enter a test suite.")),
                parameters: HashMap::new(),
                is_new: true,
            }
        );

        let test_configuration_resource = LocalResource::new(move || {
            let mut carl = globals.client.clone();
            async move {
                if let Ok(configuration) = carl.viper.get_viper_run_descriptor(test_id).await {
                    test_configuration.update(|user_configuration| {
                        user_configuration.name = UserInputValue::Right(configuration.name.value().to_string());
                        user_configuration.source = UserInputValue::Right(configuration.source.to_string());
                        user_configuration.suite = UserInputValue::Right(configuration.suite.to_string());

                        let mut parameters: HashMap<String, UserInputValue> = HashMap::new();

                        for (key, value) in configuration.parameters {

                            let value = match value {
                                ViperRunParameterValue::Boolean(boolean) => boolean.to_string(),
                                ViperRunParameterValue::Number(number) => number.to_string(),
                                ViperRunParameterValue::Text(text) => text,
                            };
                            parameters.insert(
                                key.inner,
                                UserInputValue::Right(value)
                            );
                        }
                    })
                }
            }
        });

        let is_valid_test_configuration = Memo::new(move |_| {
            test_configuration.with(|config| config.is_valid())
        });

        (test_configuration, test_configuration_resource, is_valid_test_configuration)
    };

    let _test_id = Memo::new(move |_| params.with(|params| {
        params.get("id")
            .and_then(|id| ViperRunId::try_from(id.as_str()).ok())
    }).unwrap_or_else(ViperRunId::random));

    let test_id_string = create_read_slice(test_configuration, |config| config.id.to_string());

    let breadcrumbs = Signal::derive(move || {
        let test_id = test_id_string.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Tests", "tests"),
            Breadcrumb::new(&test_id, format!("{test_id}/configure")),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = test_configuration.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = vec![
        opendut_lea_components::tabs::Tab { title: String::from("General"), href: String::from("general") },
        opendut_lea_components::tabs::Tab { title: String::from("Parameters"), href: String::from("parameters") },
    ];
    let active_tab = use_active_tab::<TabIdentifier>();

    view! {
        <BasePageContainer
            title="Configure Test"
            subtitle
            breadcrumbs
            controls=view! { <Controls configuration=test_configuration is_valid_test_configuration=is_valid_test_configuration.into() /> }
        >
            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                { move || match active_tab.get() {
                    TabIdentifier::General => view! { <GeneralTab test_configuration /> }.into_any(),
                    TabIdentifier::Parameters => view! { <ParameterTab /> }.into_any(),
                }}
            </Tabs>
        </BasePageContainer>
    }
}
