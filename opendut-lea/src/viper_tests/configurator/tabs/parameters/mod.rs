mod boolean;
mod number;
mod text;

use leptos::prelude::*;
use opendut_model::viper::ViperParameterDescriptor;
use crate::app::use_app_globals;
use crate::viper_tests::configurator::tabs::parameters::boolean::BooleanParameterInput;
use crate::viper_tests::configurator::tabs::parameters::number::NumberParameterInput;
use crate::viper_tests::configurator::tabs::parameters::text::TextParameterInput;
use crate::viper_tests::configurator::types::{SourceSelection, UserViperTestRunDescriptor, ViperBindingValueInput};

#[component]
pub fn ParametersTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let globals = use_app_globals();

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
                        .expect("Failed to request the viper test suite descriptor.");

                    Some(test_suite_descriptor.parameters)
                } else {
                    None
                }
            }
        })
    };

    view! {
        <Suspense>
            { move || Suspend::new(async move {
                let parameters = parameters.await;

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
                                ViperParameterDescriptor::NumberParameter { name, info, default, min, max } => {
                                    view! {
                                        <NumberParameterInput
                                            name=name.to_string()
                                            display_name=info.display_name
                                            description=info.description
                                            default
                                            min
                                            max
                                        />
                                    }.into_any()
                                }
                                ViperParameterDescriptor::TextParameter { name, info, default, max } => {
                                    view! {
                                        <TextParameterInput
                                            getter=test_run_getter
                                            setter=test_run_setter
                                            name=name.to_string()
                                            display_name=info.display_name
                                            description=info.description
                                            default
                                            max
                                        />
                                    }.into_any()
                                }
                            }
                        }
                    />
                }
            })}
        </Suspense>
    }
}
