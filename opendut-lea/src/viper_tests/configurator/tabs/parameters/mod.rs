mod boolean;
mod number;
mod text;

use leptos::prelude::*;
use opendut_model::viper::{ViperParameterDescriptor, ViperParameterDescriptors};
use crate::viper_tests::configurator::tabs::parameters::boolean::BooleanParameterInput;
use crate::viper_tests::configurator::tabs::parameters::number::NumberParameterInput;
use crate::viper_tests::configurator::tabs::parameters::text::TextParameterInput;
use crate::viper_tests::configurator::types::{UserViperTestRunDescriptor, ViperBindingValueInput};

#[component]
pub fn ParametersTab(
    viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    parameters: Option<ViperParameterDescriptors>,
) -> impl IntoView {

    view! {
        <For
            each=move || parameters.clone().unwrap_or_default()
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

                match parameter_descriptor {
                    ViperParameterDescriptor::BooleanParameter { name, info, default } => {
                        view! {
                            <BooleanParameterInput
                                getter=test_run_getter
                                setter=test_run_setter
                                name=name.to_string()
                                display_name=info.display_name
                                description=info.description
                                default
                            />
                        }.into_any()
                    }
                    ViperParameterDescriptor::NumberParameter { .. } => {
                        view! {
                            <NumberParameterInput
                                parameter_descriptor
                                getter=test_run_getter
                                setter=test_run_setter
                            />
                        }.into_any()
                    }
                    ViperParameterDescriptor::TextParameter { .. } => {
                        view! {
                            <TextParameterInput
                                parameter_descriptor
                                getter=test_run_getter
                                setter=test_run_setter
                            />
                        }.into_any()
                    }
                }
            }
        />
    }
}
