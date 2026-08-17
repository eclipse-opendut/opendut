use leptos::prelude::*;

use opendut_model::viper::{IllegalViperTestName, ViperTestName};
use crate::components::{UserInput, UserInputValue};
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

#[component]
pub fn ViperTestNameInput(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let (getter, setter) = create_slice(
        viper_test_run_descriptor,
        |config| {
            Clone::clone(&config.name)
        },
        |config, input| {
            config.name = input;
        }
    );

    let validator = |input: String| {
        match ViperTestName::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(source) => {
                match source {
                    IllegalViperTestName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A VIPER test name must be at least {expected} characters long."), value)
                        }
                        else {
                            UserInputValue::Both("Enter a valid VIPER test name.".to_string(), value)
                        }
                    }
                    IllegalViperTestName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A VIPER test name must be at most {expected} characters long."), value)
                    },
                    IllegalViperTestName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The VIPER test name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalViperTestName::InvalidCharacter { value } => {
                        UserInputValue::Both("The VIPER test name contains invalid characters.".to_string(), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Name"
            placeholder="MyAwesomeViperTest"
            validator
        />
    }
}
