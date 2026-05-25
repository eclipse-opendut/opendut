use leptos::prelude::*;

use opendut_model::secret::{IllegalSecretName, SecretName};
use crate::components::{UserInput, UserInputValue};
use crate::secrets::configurator::types::UserSecretConfiguration;

#[component]
pub fn SecretNameInput(secret_configuration: RwSignal<UserSecretConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(secret_configuration,
        |config| {
            Clone::clone(&config.name)
        },
        |config, input| {
            config.name = input;
        }
    );

    let validator = |input: String| {
        match SecretName::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    IllegalSecretName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A secret name must be at least {expected} characters long."), value)
                        }
                        else {
                            UserInputValue::Both("Enter a valid secret name.".to_string(), value)
                        }
                    }
                    IllegalSecretName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A secret name must be at most {expected} characters long."), value)
                    },
                    IllegalSecretName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The secret name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalSecretName::InvalidCharacter { value } => {
                        UserInputValue::Both("The secret name contains invalid characters.".to_string(), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Secret Name"
            placeholder="MyGitToken"
            validator
        />
    }
}
