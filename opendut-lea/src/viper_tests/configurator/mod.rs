use crate::api::viper::{get_viper_test_run_descriptor, get_viper_test_suite_parameters, list_viper_source_descriptors};
use std::collections::HashMap;
use leptos::prelude::*;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::{BasePageContainer, Breadcrumb, ErrorMessage, ErrorText, LoadingSpinner, UserInputError, UserInputValue};
use opendut_lea_components::tabs::{Tab, Tabs};
use opendut_model::viper::{ViperTestId};
use crate::api::clusters::list_cluster_descriptors;
use crate::api::peers::list_peer_descriptors;
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

    let viper_test_run_descriptor = get_viper_test_run_descriptor(Signal::from(viper_test_id), None);

    Effect::new(move |_| {
        if let Some(Ok(Some(descriptor))) = viper_test_run_descriptor.get() {
            user_test_run_descriptor.update(|configuration| configuration.apply_descriptor(descriptor));
        }
    });

    let viper_source = create_read_slice(
        user_test_run_descriptor,
        |descriptor| Clone::clone(&descriptor.viper_source),
    );

    let viper_sources_resource = list_viper_source_descriptors(None);

    let test_suite_parameters = get_viper_test_suite_parameters(
        Signal::derive(move || match viper_source.get() {
            SourceSelection::Left(_) => None,
            SourceSelection::Right(source_id) | SourceSelection::Both(_, source_id) => Some(source_id),
        }),
        None,
    );

    let parameters = {
        LocalResource::new(move || {
            async move {
                match test_suite_parameters.await {
                    Ok(Some(test_suite_parameters)) => {
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
                    Err(error) => Err(SourceFetchError::RequestFailed(error)),
                    Ok(None) => Err(SourceFetchError::NoSourceSelected),
                }
            }
        })
    };

    let peers = list_peer_descriptors(None);
    let clusters = list_cluster_descriptors(None);

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
                    match viper_test_run_descriptor.await {
                        Ok(_) => {
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
                                                on_retry=move || test_suite_parameters.refetch()
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
                                on_retry=move || viper_test_run_descriptor.refetch()
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
    RequestFailed(ErrorText),
}

impl SourceFetchError {
    pub fn to_error_view(&self, on_retry: Callback<()>) -> AnyView {
        match self {
            Self::NoSourceSelected => view! {
                <p class="help has-text-danger">
                    "Select a VIPER source first."
                </p>
            }.into_any(),
            Self::RequestFailed(error) => view! {
                <ErrorMessage text=error.clone() on_retry />
            }.into_any(),
        }
    }
}
