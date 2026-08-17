mod boolean;
mod number;
mod text;

use leptos::prelude::*;
use opendut_carl_api::carl::viper::GetViperTestSuiteParametersError;
use opendut_model::viper::{ViperParameterDescriptor, ViperParameterDescriptors, ViperSourceId};
use crate::viper_tests::configurator::SourceFetchError;
use crate::viper_tests::configurator::tabs::parameters::boolean::BooleanParameterInput;
use crate::viper_tests::configurator::tabs::parameters::number::NumberParameterInput;
use crate::viper_tests::configurator::tabs::parameters::text::TextParameterInput;
use crate::viper_tests::configurator::types::{UserViperTestRunDescriptor, ViperBindingValueInput};

#[component]
pub fn ParametersTab(
    viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    parameter_result: Result<ViperParameterDescriptors, SourceFetchError>,
) -> impl IntoView {

    match parameter_result {
        Ok(parameters) => {
            view! {
                <For
                    each=move || parameters.clone()
                    key=|parameter_descriptor| Clone::clone(parameter_descriptor.name())
                    children=move |parameter_descriptor| {

                        let (test_run_getter, test_run_setter) = viper_test_run_descriptor.split();

                        let test_run_getter = {
                            let parameter_descriptor = Clone::clone(&parameter_descriptor);
                            Signal::derive(move || {
                                test_run_getter.get().parameters.get(parameter_descriptor.name()).cloned()
                            })
                        };

                        let test_run_setter = {
                            let parameter_descriptor = Clone::clone(&parameter_descriptor);
                            SignalSetter::map(move |value: ViperBindingValueInput| {
                                test_run_setter.update(|test_run| {
                                    test_run.parameters.insert(Clone::clone(parameter_descriptor.name()), value);
                                })
                            })
                        };

                        let use_default_value = {
                            let has_value = matches!(test_run_getter.get_untracked(), Some(ViperBindingValueInput::Right(None)));
                            RwSignal::new(has_value)
                        };

                        Effect::new(move || {
                            if use_default_value.get() {
                                test_run_setter.set(ViperBindingValueInput::Right(None));
                            }
                        });

                        let cloned_parameter_descriptor = Clone::clone(&parameter_descriptor);

                        match parameter_descriptor {
                            ViperParameterDescriptor::BooleanParameter { name, info, default } => {
                                let default = default.unwrap_or(false);
                                view! {
                                    <BooleanParameterInput
                                        getter=test_run_getter
                                        setter=test_run_setter
                                        name=name.to_string()
                                        display_name=info.display_name
                                        description=info.description
                                        use_default_value
                                        default_value=default
                                    />
                                    <hr />
                                }.into_any()
                            }
                            ViperParameterDescriptor::NumberParameter { default, .. } => {
                                view! {
                                    <NumberParameterInput
                                        parameter_descriptor
                                        getter=test_run_getter
                                        setter=test_run_setter
                                        use_default_value
                                        default_value=default
                                    />
                                    <hr />
                                }.into_any()
                            }
                            ViperParameterDescriptor::TextParameter { default, .. } => {
                                view! {
                                    <TextParameterInput
                                        parameter_descriptor=cloned_parameter_descriptor
                                        getter=test_run_getter
                                        setter=test_run_setter
                                        use_default_value
                                        default_value=default
                                    />
                                    <hr />
                                }.into_any()
                            }
                        }
                    }
                />
            }.into_any()
        }
        Err(fetch_error) => {
            match fetch_error {
                SourceFetchError::NoSourceSelected => {
                    view! { <p class="help has-text-danger"> "No VIPER source selected." </p> }.into_any()
                }
                SourceFetchError::GetParameterError(error) => {
                    let create_source_href = move |id: &ViperSourceId| {
                        format!("/viper_sources/{id}/configure/general")
                    };
                    match error {
                        GetViperTestSuiteParametersError::SourceNotFound { .. } => {
                            view! {
                                <p class="help has-text-danger"> "Selected VIPER source not found." </p>
                            }.into_any()
                        },
                        GetViperTestSuiteParametersError::Compilation { source_id, source_name } => {
                            let source_name = source_name.to_string();
                            let source_href = create_source_href(&source_id);
                            view! {
                                <p class="help has-text-danger">
                                    "Compilation failed for the selected VIPER Source. Please check the Source "
                                    <a href=source_href> {source_name} </a>.
                                </p>
                            }.into_any()
                        },
                        GetViperTestSuiteParametersError::ViperRuntime { source_id, source_name } => {
                            let source_name = source_name.to_string();
                            let source_href = create_source_href(&source_id);
                            view! {
                                <p class="help has-text-danger">
                                    "Error while initializing VIPER runtime for VIPER test source "
                                    <a href=source_href> {source_name} </a>.
                                </p>
                            }.into_any()
                        },
                        GetViperTestSuiteParametersError::Internal { source_id, source } => {
                            let source_href = create_source_href(&source_id);
                            view! {
                                <p class="help has-text-danger">
                                    "An internal error occurred while fetching the VIPER test suite descriptor for the "
                                    <a href=source_href> selected source </a>:
                                    <br />
                                    {source}
                                </p>
                            }.into_any()
                        },
                    }
                }
            }
        }
    }
}
