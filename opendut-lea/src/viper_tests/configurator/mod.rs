use crate::api::viper::list_viper_source_descriptors;
use crate::util::client_error_description;
use std::collections::HashMap;
use leptos::prelude::*;
use tracing::error;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_carl_api::carl::ClientError;
use opendut_carl_api::carl::viper::{CompilationErrorKind, GetViperTestRunDescriptorError, GetViperTestSuiteParametersError};
use opendut_lea_components::{BasePageContainer, Breadcrumb, ErrorMessage, ErrorText, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::cluster::ClusterDescriptor;
use opendut_model::peer::PeerDescriptor;
use opendut_model::viper::{ViperParameterName, ViperTestId, ViperTestRunDescriptor};
use crate::api::clusters::list_cluster_descriptors;
use crate::api::peers::{list_peer_descriptors, list_peer_states};
use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::viper_tests::configurator::components::Controls;
use crate::viper_tests::configurator::tabs::{GeneralTab, SourceTab, ParametersTab, TabIdentifier, PeerTab};
use crate::viper_tests::configurator::types::{PeerSelection, SourceSelection, UserViperTestRunDescriptor, ViperBindingValueInput};

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

    let user_test_run_descriptor = RwSignal::new(
        UserViperTestRunDescriptor {
            id: viper_test_id,
            name: UserInputValue::Left(UserInputError::from("Enter a valid VIPER test name.")),
            viper_source: SourceSelection::Left(String::from("Select a VIPER test source.")),
            peer: PeerSelection::Left(String::from("Select a peer")),
            parameters: HashMap::new(),
        }
    );

    let viper_test_run_descriptor_resource = {
        let carl = globals.client.clone();
        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                let descriptor = match carl.viper.get_viper_test_run_descriptor(viper_test_id).await {
                    Ok(descriptor) => descriptor,
                    Err(ClientError::UsageError(GetViperTestRunDescriptorError::TestNotFound { .. })) => {
                        return Ok(());
                    }
                    Err(error) => {
                        error!("Failed to request the VIPER test run descriptor <{viper_test_id}>: {error}");

                        let error_description = client_error_description(
                            &error,
                            "The VIPER test run descriptor could not be loaded. Please check your connection and try again.",
                        );

                        return Err(ErrorText {
                            headline: String::from("Unable to load VIPER test run descriptor"),
                            description: error_description,
                        });
                    }
                };

                user_test_run_descriptor.update(|user_configuration| {
                    let ViperTestRunDescriptor { id: _, name, source: viper_source, peer, parameters } = descriptor;

                    user_configuration.name = UserInputValue::Right(name.value().to_owned());
                    user_configuration.viper_source = SourceSelection::Right(viper_source);
                    user_configuration.peer = PeerSelection::Right(peer);

                    let mut configured_parameters: HashMap<ViperParameterName, ViperBindingValueInput> = HashMap::new();

                    for (key, value) in parameters {
                        configured_parameters.insert(
                            key,
                            ViperBindingValueInput::Right(value)
                        );
                    }

                    user_configuration.parameters = configured_parameters;
                });

                Ok(())
            }
        })
    };

    let viper_source = create_read_slice(
        user_test_run_descriptor,
        |descriptor| Clone::clone(&descriptor.viper_source),
    );

    let viper_sources_resource = list_viper_source_descriptors(None);

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
                        Ok(test_suite_parameters) => {
                            user_test_run_descriptor.update(|user_configuration| {
                                let mut new_parameters = HashMap::new();

                                for parameter in test_suite_parameters.parameters.iter() {
                                    let parameter_name = parameter.name().clone();

                                    let value = user_configuration
                                        .parameters
                                        .get(&parameter_name)
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            if parameter.has_default_value() {
                                                ViperBindingValueInput::Right(None)
                                            } else {
                                                ViperBindingValueInput::Left(String::from("Please enter a value."))
                                            }
                                        });

                                    new_parameters.insert(parameter_name, value);
                                }
                                user_configuration.parameters = new_parameters;
                            });

                            Ok(test_suite_parameters.parameters)
                        }
                        Err(client_error) => {
                            match client_error {
                                ClientError::UsageError(error) => {
                                    Err(SourceFetchError::GetParameterError(error))
                                }
                                _ => {
                                    Err(SourceFetchError::RequestFailed(String::from("The VIPER test suite parameters could not be loaded. Please check your connection and try again."),))
                                }
                            }
                        }
                    }
                } else {
                    Err(SourceFetchError::NoSourceSelected)
                }
            }
        })
    };

    let peers = list_peer_descriptors(None);
    let clusters = list_cluster_descriptors();

    let viper_test_id_string = create_read_slice(user_test_run_descriptor, |config| config.id.to_string());

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
        if let UserInputValue::Right(name) = user_test_run_descriptor.get().name {
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
            ).with_is_error(Signal::derive(move || !user_test_run_descriptor.read().valid_general_tab())),

            Tab::from_title_and_href(
                String::from("VIPER Source"),
                TabIdentifier::ViperSource.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_test_run_descriptor.read().valid_viper_source_tab())),

            Tab::from_title_and_href(
                String::from("Parameters"),
                TabIdentifier::Parameters.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_test_run_descriptor.read().valid_parameters_tab())),

            Tab::from_title_and_href(
                String::from("Peer"),
                TabIdentifier::Peer.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_test_run_descriptor.read().valid_peer_tab())),
        ]
    });
    
    let on_retry_peers = Callback::new(move |()| peers.refetch());
    let on_retry_clusters = Callback::new(move |()| clusters.refetch());

    view! {
        <BasePageContainer
            title="Configure VIPER Test"
            subtitle
            breadcrumbs
            controls=view! { <Controls user_test_run_descriptor /> }
        >
            <Suspense
                fallback=move || view! { <LoadingSpinner /> }
            >
                { move || Suspend::new(async move {
                    match viper_test_run_descriptor_resource.await {
                        Ok(()) => {
                            let peers = RwSignal::new(peers.await).read_only();
                            let clusters = RwSignal::new(clusters.await).read_only();
                            let parameter_result = parameters.await;
                            let viper_sources = RwSignal::new(viper_sources_resource.await).read_only();
                            view! {
                                <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                                    { move || match active_tab.get() {
                                        TabIdentifier::General => view! { <GeneralTab user_test_run_descriptor /> }.into_any(),
                                        TabIdentifier::ViperSource => view! {
                                            <SourceTab
                                                user_test_run_descriptor
                                                viper_sources
                                                on_retry=move || viper_sources_resource.refetch()
                                            />
                                        }.into_any(),
                                        TabIdentifier::Parameters => view! {
                                            <ParametersTab
                                                user_test_run_descriptor
                                                parameter_result=parameter_result.clone()
                                                on_retry=move || parameters.refetch()
                                            />
                                        }.into_any(),
                                        TabIdentifier::Peer => view! {
                                            <PeerTab
                                                user_test_run_descriptor
                                                peers
                                                clusters
                                                on_retry_peers
                                                on_retry_clusters
                                            />
                                        }.into_any()
                                    }}
                                </Tabs>
                            }.into_any()
                        }
                        Err(error) => view! {
                            <ErrorMessage
                                text=error
                                on_retry=move || viper_test_run_descriptor_resource.refetch()
                            />
                        }.into_any(),
                    }
                })}
            </Suspense>
        </BasePageContainer>
    }
}

#[derive(Debug, Clone)]
enum SourceFetchError {
    NoSourceSelected,
    GetParameterError(GetViperTestSuiteParametersError),
    RequestFailed(String),
}

impl SourceFetchError {
    pub fn to_error_view(&self, on_retry: Callback<()>) -> AnyView {
        let description = match self {
            Self::NoSourceSelected => {
                return view! {
                    <p class="help has-text-danger">
                        "Select a VIPER source first."
                    </p>
                }.into_any();
            }
            Self::GetParameterError(error) => match error {
                GetViperTestSuiteParametersError::Compilation { kind, .. } => {
                    match kind {
                        CompilationErrorKind::FailedLoading { message, .. }
                        | CompilationErrorKind::InvalidSource { message }
                        | CompilationErrorKind::FailedInspection { message }
                        | CompilationErrorKind::PythonCompilationError { message }
                        | CompilationErrorKind::Internal { message } => {
                            format!("{error}: \n {message}")
                        }
                        CompilationErrorKind::NoSuitableSourceLoader { .. } => {
                            error.to_string()
                        }
                    }
                }
                _ => error.to_string(),
            }
            Self::RequestFailed(message) => message.to_owned(),
        };

        let error_text = ErrorText {
            headline: String::from("Unable to load VIPER parameters"),
            description,
        };

        view! { <ErrorMessage text=error_text on_retry /> }.into_any()
    }
}
