mod boolean;
mod number;
mod text;

use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use opendut_model::viper::{ViperParameterDescriptor, ViperParameterDescriptors};
use crate::viper_tests::configurator::SourceFetchError;
use crate::viper_tests::configurator::tabs::parameters::boolean::BooleanParameterInput;
use crate::viper_tests::configurator::tabs::parameters::number::NumberParameterInput;
use crate::viper_tests::configurator::tabs::parameters::text::TextParameterInput;
use crate::viper_tests::configurator::types::{UserViperTestRunDescriptor, ViperBindingValueInput};

#[component]
pub fn ParametersTab(
    user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    parameter_result: Result<ViperParameterDescriptors, SourceFetchError>,
    #[prop(into)] on_retry: Callback<()>,
) -> impl IntoView {

    let view = match parameter_result {
        Ok(parameters) => {
            if parameters.is_empty() {
                view! {
                    <p> "The selected VIPER source does not include any parameters." </p>
                }.into_any()
            } else {
                view! {
                    <For
                        each=move || parameters.clone()
                        key=|parameter_descriptor| Clone::clone(parameter_descriptor.name())
                        children=move |parameter_descriptor| {

                            let (test_run_getter, test_run_setter) = user_test_run_descriptor.split();

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
        }
        Err(fetch_error) => {
            fetch_error.to_error_view(on_retry)
        }
    };

    view! {
        <InfoMessage
            text="Set the parameters defined in your test code, to run with different configurations."
        />
        { view }
    }
}
