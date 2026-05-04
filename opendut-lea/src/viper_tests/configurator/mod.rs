use std::collections::HashMap;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::viper::{ViperBindingValue, ViperParameterDescriptor, ViperParameterName, ViperTestId, ViperTestRunDescriptor};
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_tests::configurator::components::Controls;
use crate::viper_tests::configurator::tabs::{ClusterTab, GeneralTab, SourceTab, ParametersTab, TabIdentifier};
use crate::viper_tests::configurator::types::{ClusterSelection, SourceSelection, UserViperTestRunDescriptor, ViperBindingValueInput};

mod tabs;
mod types;
mod components;

#[component(transparent)]
pub fn ViperTestConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

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

    let viper_test_run_descriptor = RwSignal::new(
        UserViperTestRunDescriptor {
            id: viper_test_id,
            name: UserInputValue::Left(UserInputError::from("Enter a valid viper test name.")),
            viper_source: SourceSelection::Left(String::from("Select a viper test source.")),
            cluster: ClusterSelection::Left(String::from("Enter a cluster.")),
            parameters: HashMap::new(),
            is_new: true,
        }
    );

    let viper_test_run_descriptor_resource = {
        let carl = globals.client.clone();
        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                if let Ok(descriptor) = carl.viper.get_viper_test_run_descriptor(viper_test_id).await {
                    viper_test_run_descriptor.update(|user_configuration| {
                        let ViperTestRunDescriptor { id: _, name, source: viper_source, cluster, parameters } = descriptor;

                        user_configuration.name = UserInputValue::Right(name.value().to_owned());
                        user_configuration.viper_source = SourceSelection::Right(viper_source);
                        user_configuration.cluster = ClusterSelection::Right(cluster);

                        let mut configured_parameters: HashMap<ViperParameterName, ViperBindingValueInput> = HashMap::new();

                        for (key, value) in parameters {
                            configured_parameters.insert(
                                key,
                                ViperBindingValueInput::Right(value)
                            );
                        }

                        user_configuration.parameters = configured_parameters;
                    })
                }
            }
        })
    };

    let viper_source = create_read_slice(
        viper_test_run_descriptor,
        |descriptor| Clone::clone(&descriptor.viper_source),
    );

    let parameters = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();
            let viper_source = viper_source.get();

            let source_id = match viper_source {
                SourceSelection::Left(_) => None,
                SourceSelection::Right(source_id) | SourceSelection::Both(_, source_id) => Some(source_id),
            };

            async move {
                if let Some(source_id) = source_id {
                    let test_suite_descriptor = carl.viper.get_viper_test_suite_parameters(source_id).await
                        .expect("Failed to request the viper test suite descriptor."); // Todo: Error-Handling

                    viper_test_run_descriptor.update(|user_configuration| {
                        let mut new_parameters: HashMap<ViperParameterName, ViperBindingValueInput> = HashMap::new();

                        for parameter in test_suite_descriptor.parameters.iter() {
                            let parameter_name = parameter.name().clone();

                            let value = user_configuration
                                .parameters
                                .get(&parameter_name)
                                .cloned()
                                .unwrap_or_else(|| default_value_for_parameter(&parameter));

                            new_parameters.insert(parameter_name, value);
                        }
                        user_configuration.parameters = new_parameters;
                    });

                    Some(test_suite_descriptor.parameters)
                } else {
                    None
                }
            }
        })
    };

    let is_valid_configuration = Memo::new(move |_| {
        viper_test_run_descriptor.with(|config| config.is_valid())
    });

    let viper_test_id_string = create_read_slice(viper_test_run_descriptor, |config| config.id.to_string());

    let breadcrumbs = Signal::derive(move || {
        let viper_test_id = viper_test_id_string.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Viper Tests", "viper_tests"),
            Breadcrumb::new(&viper_test_id, format!("{viper_test_id}/configure")),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = viper_test_run_descriptor.get().name {
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
            ).with_is_error(Signal::derive(move || !viper_test_run_descriptor.read().valid_general_tab())),

            Tab::from_title_and_href(
                String::from("Viper Source"),
                TabIdentifier::ViperSource.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_run_descriptor.read().valid_viper_source_tab())),

            Tab::from_title_and_href(
                String::from("Parameters"),
                TabIdentifier::Parameters.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_run_descriptor.read().valid_parameters_tab())),

            Tab::from_title_and_href(
                String::from("Cluster"),
                TabIdentifier::Cluster.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_run_descriptor.read().valid_cluster_tab())),
        ]
    });

    let active_tab = use_active_tab::<TabIdentifier>();

    view! {
        <BasePageContainer
            title="Configure Viper Test"
            subtitle
            breadcrumbs
            controls=view! { <Controls configuration=viper_test_run_descriptor is_valid_configuration /> }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                {
                    move || Suspend::new(async move {
                        viper_test_run_descriptor_resource.await;
                        let parameters = parameters.await;

                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                { move || match active_tab.get() {
                                    TabIdentifier::General => view! { <GeneralTab viper_test_run_descriptor /> }.into_any(),
                                    TabIdentifier::ViperSource => view! { <SourceTab viper_test_run_descriptor /> }.into_any(),
                                    TabIdentifier::Parameters => view! { <ParametersTab viper_test_run_descriptor parameters=parameters.clone() /> }.into_any(),
                                    TabIdentifier::Cluster => view! { <ClusterTab viper_test_run_descriptor /> }.into_any(),
                                }}
                            </Tabs>
                        }
                    })
                }
            </Suspense>
        </BasePageContainer>
    }
}

fn default_value_for_parameter(parameter: &ViperParameterDescriptor) -> ViperBindingValueInput {
    if parameter.has_default_value() {
        match parameter {
            ViperParameterDescriptor::BooleanParameter { default, .. } => {
                let default = default.unwrap();
                ViperBindingValueInput::Right(ViperBindingValue::BooleanValue(default))
            }
            ViperParameterDescriptor::NumberParameter { default, .. } => {
                let default = default.unwrap();
                ViperBindingValueInput::Right(ViperBindingValue::NumberValue(default))
            }
            ViperParameterDescriptor::TextParameter { default, .. } => {
                let default = default.to_owned().unwrap();
                ViperBindingValueInput::Right(ViperBindingValue::TextValue(default))
            }
        }
    } else {
        ViperBindingValueInput::Left(
            String::from("Please enter a value.")
        )
    }
}
