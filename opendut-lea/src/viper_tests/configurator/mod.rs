use std::collections::HashMap;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::viper::GetViperTestSuiteParametersError;
use opendut_lea_components::{BasePageContainer, Breadcrumb, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::viper::{ViperParameterDescriptor, ViperParameterName, ViperTestId, ViperTestRunDescriptor};
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_tests::configurator::components::Controls;
use crate::viper_tests::configurator::tabs::{ClusterTab, GeneralTab, SourceTab, ParametersTab, TabIdentifier, PeerTab};
use crate::viper_tests::configurator::types::{ClusterSelection, PeerSelection, SourceSelection, UserViperTestRunDescriptor, ViperBindingValueInput};

mod tabs;
mod types;
mod components;

#[component(transparent)]
pub fn ViperTestConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();
    let active_tab = use_active_tab::<TabIdentifier>();

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
            name: UserInputValue::Left(UserInputError::from("Enter a valid VIPER test name.")),
            viper_source: SourceSelection::Left(String::from("Select a VIPER test source.")),
            cluster: ClusterSelection::Left(String::from("Select a cluster.")),
            peer: PeerSelection::Left(String::from("Select a peer")),
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
                        let ViperTestRunDescriptor { id: _, name, source: viper_source, cluster, peer, parameters } = descriptor;

                        user_configuration.name = UserInputValue::Right(name.value().to_owned());
                        user_configuration.viper_source = SourceSelection::Right(viper_source);
                        user_configuration.cluster = ClusterSelection::Right(cluster);
                        user_configuration.peer = PeerSelection::Right(peer);
                        
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
                    let result = carl.viper.get_viper_test_suite_parameters(source_id).await;

                    match result {
                        Ok(test_suite_descriptor) => {
                            viper_test_run_descriptor.update(|user_configuration| {
                                let mut new_parameters: HashMap<ViperParameterName, ViperBindingValueInput> = HashMap::new();

                                for parameter in test_suite_descriptor.parameters.iter() {
                                    let parameter_name = parameter.name().clone();

                                    let value = user_configuration
                                        .parameters
                                        .get(&parameter_name)
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            match parameter {
                                                ViperParameterDescriptor::BooleanParameter { .. } => {
                                                    // Todo: Remove this match arm, when backend/VIPER forces default parameters for `BooleanParameter`.
                                                    ViperBindingValueInput::Right(None)
                                                }
                                                _ if parameter.has_default_value() => {
                                                    ViperBindingValueInput::Right(None)
                                                }
                                                _ => {
                                                    ViperBindingValueInput::Left(String::from("Please enter a value."))
                                                }
                                            }
                                        });

                                    new_parameters.insert(parameter_name, value);
                                }
                                user_configuration.parameters = new_parameters;
                            });

                            Ok(test_suite_descriptor.parameters)
                        },
                        Err(client_error) => {
                            match client_error {
                                ClientError::UsageError(error) => {
                                    Err(SourceFetchError::GetParameterError(error))
                                }
                                _ => panic!("Failed to request the viper test suite descriptor.")
                            }
                        }
                    }
                } else {
                    Err(SourceFetchError::NoSourceSelected)
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
        let active_tab = active_tab.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("VIPER Tests", "viper_tests"),
            Breadcrumb::new("Configure VIPER Test", format!("{viper_test_id}/configure/{}", active_tab.as_str())),
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
                String::from("VIPER Source"),
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

            Tab::from_title_and_href(
                String::from("Peer"),
                TabIdentifier::Peer.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !viper_test_run_descriptor.read().valid_cluster_tab())),
        ]
    });
    
    view! {
        <BasePageContainer
            title="Configure VIPER Test"
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
                        let parameter_result = parameters.await;
                        view! {
                            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                { move || match active_tab.get() {
                                    TabIdentifier::General => view! { <GeneralTab viper_test_run_descriptor /> }.into_any(),
                                    TabIdentifier::ViperSource => view! { <SourceTab viper_test_run_descriptor /> }.into_any(),
                                    TabIdentifier::Parameters => view! { <ParametersTab viper_test_run_descriptor parameter_result=parameter_result.clone() /> }.into_any(),
                                    TabIdentifier::Cluster => view! { <ClusterTab viper_test_run_descriptor /> }.into_any(),
                                    TabIdentifier::Peer => view! { <PeerTab /> }.into_any()
                                }}
                            </Tabs>
                        }
                    })
                }
            </Suspense>
        </BasePageContainer>
    }
}

#[derive(Debug, Clone)]
enum SourceFetchError {
    NoSourceSelected,
    GetParameterError(GetViperTestSuiteParametersError),
}
