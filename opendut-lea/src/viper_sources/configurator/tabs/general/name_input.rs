use leptos::prelude::*;
use opendut_model::viper::{InvalidViperTestSuiteIdentifierErrorKind, ViperTestSuiteIdentifier};
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
        match ViperTestSuiteIdentifier::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(source) => {
                match source.kind {
                    InvalidViperTestSuiteIdentifierErrorKind::Empty => {
                        UserInputValue::Both("Enter a VIPER source name.".to_string(), source.value)
                    }
                    InvalidViperTestSuiteIdentifierErrorKind::IllegalTestSuiteIdentifierCharacter { character } => {
                        UserInputValue::Both(format!("The VIPER source name contains an invalid character: '{character}'"), source.value)
                    }
                    _ => {
                        UserInputValue::Both("The VIPER source name is invalid.".to_string(), source.value)
                    }
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
