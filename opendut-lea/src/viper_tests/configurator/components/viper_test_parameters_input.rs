use leptos::prelude::*;

use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn ViperTestParametersInput(viper_test_configuration: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_test_configuration,
        |config| {
            Clone::clone(&config.parameters)
        },
        |config, input| {
            config.parameters = input;
        }
    );

    view! {
        { move || format!("{:#?}", getter.get()) }
    }
}
