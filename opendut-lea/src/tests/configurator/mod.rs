use std::collections::HashMap;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, UserInputError, UserInputValue};
use opendut_model::viper::{ViperRunId, ViperRunParameterValue};
use crate::app::use_app_globals;
use crate::components::tabs::{Tab, TabIdentifier, TabState, ConfiguratorTabs};
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::tests::configurator::tabs::{GeneralTab, ParameterTab};
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
                source: UserInputValue::Left(String::from("Enter a valid test name.")),
                suite: UserInputValue::Left(String::from("Enter a valid test name.")),
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

    let test_id = Memo::new(move |_| params.with(|params| {
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

    let tabs = vec![
        Tab::new(TabIdentifier::General, String::from("General"), TabState::Normal, || view! {<GeneralTab />}.into_any()),
        Tab::new(TabIdentifier::Parameters, String::from("Parameters"), TabState::Normal, || view! {<ParameterTab />}.into_any()),
    ];

    view! {
        <BasePageContainer
            title="Configure Test"
            subtitle=String::new() // Todo: Set subtitle (name)
            breadcrumbs
            controls=view! { }
        >
            <ConfiguratorTabs tabs />
        </BasePageContainer>
    }
}
