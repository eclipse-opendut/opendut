use leptos::prelude::*;
use opendut_lea_components::{UserInput, UserInputValue};
use opendut_model::viper::{InvalidTextParameterValueErrorKind, ViperBindingValue, ViperParameterDescriptor, ViperParameterInfo};
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn TextParameterInput(
    parameter_descriptor: ViperParameterDescriptor,
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
) -> impl IntoView {

    let name = parameter_descriptor.name().to_string();
    let ViperParameterInfo { display_name, description } = parameter_descriptor.info().to_owned();

    let getter = Signal::derive(move || {
        let getter_value = getter.get()
            .unwrap_or(ViperBindingValueInput::Right(None));

        let to_text = |value| match value {
            Some(ViperBindingValue::TextValue(text)) => Ok(text),
            None => Ok(String::new()),
            _ => Err(String::from("Invalid parameter type, expected a text parameter")),
        };

        match getter_value {
            ViperBindingValueInput::Left(error) => {
                UserInputValue::Left(error)
            }

            ViperBindingValueInput::Right(value) => {
                match to_text(value) {
                    Ok(text) => UserInputValue::Right(text),
                    Err(error) => UserInputValue::Left(error),
                }
            }

            ViperBindingValueInput::Both(error, value) => {
                match to_text(value) {
                    Ok(text) => UserInputValue::Both(error, text),
                    Err(_) => UserInputValue::Left(error),
                }
            }
        }
    });

    let setter = SignalSetter::map(move |value: UserInputValue| {
        let value = match value {
            UserInputValue::Left(error) => {
                ViperBindingValueInput::Left(error)
            }
            UserInputValue::Right(value) => {
                let value = ViperBindingValue::TextValue(value);
                ViperBindingValueInput::Right(Some(value))
            }
            UserInputValue::Both(error, value) => {
                let value = ViperBindingValue::TextValue(value);
                ViperBindingValueInput::Both(error, Some(value))
            }
        };

        setter.set(value);
    });

    let validator = move |input: String| {
        match parameter_descriptor.validate_text_parameter(&input) {
            Ok(_) => UserInputValue::Right(input),
            Err(error) => {
                match error.kind {
                    InvalidTextParameterValueErrorKind::Empty => {
                        UserInputValue::Both(String::from("Please enter a value."), input)
                    }
                    InvalidTextParameterValueErrorKind::TooLong { expected, actual: _ } => {
                        UserInputValue::Both(
                            format!("The text parameter must be at most {expected} characters long."),
                            input,
                        )
                    }
                    InvalidTextParameterValueErrorKind::InvalidType { expected, actual } => {
                        UserInputValue::Both(
                            format!("The parameter must be a {expected} (actual: {actual})."),
                            input,
                        )
                    }
                }
            }
        }
    };

    view! {
        <UserInput
            getter
            setter
            validator
            label=display_name.unwrap_or_else(|| name)
            placeholder="Text Parameter"
            description
        />
    }
}
