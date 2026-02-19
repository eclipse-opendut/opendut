use leptos::prelude::*;

use crate::components::{UserInput, UserInputValue};
use crate::viper_tests::configurator::types::UserViperTestConfiguration;

#[component]
pub fn ViperTestSuiteSelector(viper_test_configuration: RwSignal<UserViperTestConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_test_configuration,
        |config| {
            Clone::clone(&config.viper_test_suite)
        },
        |config, input| {
            config.viper_test_suite = input;
        }
    );

    let validator = |input: String| {
        if input.trim().is_empty() {
            UserInputValue::Both(String::from("Enter a viper test suite"), input)
        } else {
            UserInputValue::Right(input)
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Viper Test Suite"
            placeholder="script.py"
            validator
        />
    }
}
