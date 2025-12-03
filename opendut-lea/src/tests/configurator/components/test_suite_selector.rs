use leptos::prelude::*;

use crate::components::{UserInput, UserInputValue};
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn TestSuiteSelector(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(test_configuration,
        |config| {
            Clone::clone(&config.suite)
        },
        |config, input| {
            config.suite = input;
        }
    );

    let validator = |input: String| {
        if input.trim().is_empty() {
            UserInputValue::Both(String::from("Enter a test suite"), input)
        } else {
            UserInputValue::Right(input)
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Test Suite"
            placeholder="script.py"
            validator=validator
        />
    }
}
