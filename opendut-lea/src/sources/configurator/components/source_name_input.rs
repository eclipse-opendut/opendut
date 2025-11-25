use leptos::prelude::*;

use opendut_model::viper::{IllegalViperSourceName, ViperSourceName};
use crate::components::{UserInput, UserInputValue};
use crate::sources::configurator::types::UserSourceConfiguration;

#[component]
pub fn SourceNameInput(source_configuration: RwSignal<UserSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(source_configuration,
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
                            UserInputValue::Both(format!("A source name must be at least {expected} characters long."), value)
                        }
                        else {
                            UserInputValue::Both("Enter a valid source name.".to_string(), value)
                        }
                    }
                    IllegalViperSourceName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A source name must be at most {expected} characters long."), value)
                    },
                    IllegalViperSourceName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The source name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalViperSourceName::InvalidCharacter { value } => {
                        UserInputValue::Both("The source name contains invalid characters.".to_string(), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Source Name"
            placeholder="MyAwesomeViperSource"
            validator=validator
        />
    }
}
