use leptos::prelude::*;

use opendut_model::viper::{IllegalViperRunName, ViperRunName};
use crate::components::{UserInput, UserInputValue};
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn TestNameInput(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(test_configuration,
        |config| {
            Clone::clone(&config.name)
        },
        |config, input| {
            config.name = input;
        }
    );

    let validator = |input: String| {
        match ViperRunName::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    IllegalViperRunName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A test name must be at least {expected} characters long."), value)
                        }
                        else {
                            UserInputValue::Both("Enter a valid test name.".to_string(), value)
                        }
                    }
                    IllegalViperRunName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A test name must be at most {expected} characters long."), value)
                    },
                    IllegalViperRunName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The test name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalViperRunName::InvalidCharacter { value } => {
                        UserInputValue::Both("The test name contains invalid characters.".to_string(), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Test Name"
            placeholder="MyAwesomeViperTest"
            validator=validator
        />
    }
}
