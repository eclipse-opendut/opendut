use std::collections::HashMap;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::viper::{ViperTestRunDescriptor, ViperTestId, ViperTestParameterValue};
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_tests::configurator::components::Controls;
use crate::viper_tests::configurator::tabs::{ClusterTab, GeneralTab, SourceTab, SuiteTab, TabIdentifier};
use crate::viper_tests::configurator::types::{ClusterSelection, SourceSelection, UserViperTestConfiguration};

mod tabs;
mod types;
mod components;

#[component(transparent)]
pub fn ViperTestConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let (viper_test_configuration, viper_test_configuration_resource, is_valid_configuration) = {
        let viper_test_id = {
            let viper_test_id = params.with_untracked(|params| {
                params.get("id").and_then(|id| ViperTestId::try_from(id.as_str()).ok())
            });
            match viper_test_id {
                None => {
                    let use_navigate = use_navigate();
                    navigate_to(WellKnownRoutes::ErrorPage {
                        title: String::from("Invalid ViperTestId"),
                        text: String::from("Could not parse the provided value as ViperTestId!"),
                        details: None,
                    }, use_navigate);
                    ViperTestId::random()
                }
                Some(viper_test_id) => {
                    viper_test_id
                }
            }
        };

        let viper_test_configuration = RwSignal::new(
            UserViperTestConfiguration {
                id: viper_test_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid viper test name.")),
                viper_source: SourceSelection::Left(String::from("Select a viper test source.")),
                viper_test_suite: UserInputValue::Left(String::from("Enter a viper test suite.")),
                cluster: ClusterSelection::Left(String::from("Enter a cluster.")),
                parameters: HashMap::new(),
                is_new: true,
            }
        );

        let viper_test_configuration_resource = LocalResource::new(move || {
            let mut carl = globals.client.clone();
            async move {
                if let Ok(configuration) = carl.viper.get_viper_test_run_descriptor(viper_test_id).await {
                    viper_test_configuration.update(|user_configuration| {
                        let ViperTestRunDescriptor { id: _, name, source: viper_source, suite: viper_test_suite, cluster, parameters } = configuration;

                        user_configuration.name = UserInputValue::Right(name.value().to_owned());
                        user_configuration.viper_source = SourceSelection::Right(viper_source);
                        user_configuration.viper_test_suite = UserInputValue::Right(viper_test_suite.to_string());
                        user_configuration.cluster = ClusterSelection::Right(cluster);

                        let mut configured_parameters: HashMap<String, UserInputValue> = HashMap::new();

                        for (key, value) in parameters { //TODO this loop doesn't do anything?

                            let value = match value {
                                ViperTestParameterValue::Boolean(boolean) => boolean.to_string(),
                                ViperTestParameterValue::Number(number) => number.to_string(),
                                ViperTestParameterValue::Text(text) => text,
                            };
                            configured_parameters.insert(
                                key.inner,
                                UserInputValue::Right(value)
                            );
                        }
                    })
                }
            }
        });

        let is_valid_configuration = Memo::new(move |_| {
            viper_test_configuration.with(|config| config.is_valid())
        });

        (viper_test_configuration, viper_test_configuration_resource, is_valid_configuration)
    };

    let viper_test_id_string = create_read_slice(viper_test_configuration, |config| config.id.to_string());

    let breadcrumbs = Signal::derive(move || {
        let viper_test_id = viper_test_id_string.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Viper Tests", "viper_tests"),
            Breadcrumb::new(&viper_test_id, format!("{viper_test_id}/configure")),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = viper_test_configuration.get().name {
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
            ).with_is_error(Signal::derive(move || !viper_test_configuration.read().valid_general_tab())),

            Tab::from_title_and_href(
                String::from("Viper Source"),
                TabIdentifier::ViperSource.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_configuration.read().valid_viper_source_tab())),

            Tab::from_title_and_href(
                String::from("Viper Test Suite"),
                TabIdentifier::ViperTestSuite.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_configuration.read().valid_viper_test_suite_tab())),

            Tab::from_title_and_href(
                String::from("Cluster"),
                TabIdentifier::Cluster.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_configuration.read().valid_cluster_tab())),
        ]
    });

    let active_tab = use_active_tab::<TabIdentifier>();

    view! {
        <BasePageContainer
            title="Configure Viper Test"
            subtitle
            breadcrumbs
            controls=view! { <Controls configuration=viper_test_configuration is_valid_configuration /> }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                {
                    move || Suspend::new(async move {
                        viper_test_configuration_resource.await;

                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                { move || match active_tab.get() {
                                    TabIdentifier::General => view! { <GeneralTab viper_test_configuration /> }.into_any(),
                                    TabIdentifier::ViperSource => view! { <SourceTab viper_test_configuration /> }.into_any(),
                                    TabIdentifier::ViperTestSuite => view! { <SuiteTab viper_test_configuration /> }.into_any(),
                                    TabIdentifier::Cluster => view! { <ClusterTab viper_test_configuration /> }.into_any(),
                                }}
                            </Tabs>
                        }
                    })
                }
            </Suspense>
        </BasePageContainer>
    }
}
