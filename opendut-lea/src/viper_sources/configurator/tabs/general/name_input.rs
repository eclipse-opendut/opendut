use leptos::prelude::*;

use opendut_model::viper::{IllegalViperSourceName, ViperSourceName};
use crate::components::{UserInput, UserInputValue};
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceNameInput(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_source_configuration,
        |config| {
            Clone::clone(&config.name)
        },
        |config, input| {
            config.name = input;
        }
    );

    let validator = |input: String| {
        match ViperSourceName::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    IllegalViperSourceName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A VIPER source name must be at least {expected} characters long."), value)
                        }
                        else {
                            UserInputValue::Both("Enter a VIPER viper source name.".to_string(), value)
                        }
                    }
                    IllegalViperSourceName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A VIPER source name must be at most {expected} characters long."), value)
                    },
                    IllegalViperSourceName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The VIPER source name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalViperSourceName::InvalidCharacter { value } => {
                        UserInputValue::Both("The VIPER source name contains invalid characters.".to_string(), value)
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
            placeholder="MyAwesomeViperSource"
            validator
        />
    }
}
