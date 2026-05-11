use leptos::prelude::*;
use opendut_lea_components::{InputType, UserInput, UserInputValue};
use opendut_model::viper::{InvalidNumberParameterValueErrorKind, ViperBindingValue, ViperParameterDescriptor, ViperParameterInfo};
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn NumberParameterInput(
    parameter_descriptor: ViperParameterDescriptor,
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
    use_default_value: Option<RwSignal<bool>>,
    default_value: Option<i64>,
) -> impl IntoView {

    let name = parameter_descriptor.name().to_string();
    let ViperParameterInfo { display_name, description } = parameter_descriptor.info().to_owned();

    let getter = Signal::derive(move || {
        let getter_value = getter.get()
            .unwrap_or(ViperBindingValueInput::Right(None));

        let to_number_text = |value| match value {
            Some(ViperBindingValue::NumberValue(number)) => Ok(number.to_string()),
            None => Ok(String::new()),
            _ => Err(String::from("Invalid parameter type, expected a number parameter")),
        };

        match getter_value {
            ViperBindingValueInput::Left(error) => {
                UserInputValue::Left(error)
            }

            ViperBindingValueInput::Right(value) => {
                match to_number_text(value) {
                    Ok(number) => UserInputValue::Right(number),
                    Err(error) => UserInputValue::Left(error),
                }
            }

            ViperBindingValueInput::Both(error, value) => {
                match to_number_text(value) {
                    Ok(number) => UserInputValue::Both(error, number),
                    Err(_) => UserInputValue::Left(error),
                }
            }
        }
    });

    let input_setter = SignalSetter::map(move |value: UserInputValue| {
        let value = match value {
            UserInputValue::Left(error) => {
                ViperBindingValueInput::Left(error)
            }
            UserInputValue::Right(input) => {
                match parse_number_binding_value(&input) {
                    Some(value) => ViperBindingValueInput::Right(Some(value)),
                    None => ViperBindingValueInput::Left(
                        String::from("Invalid parameter type, expected a number parameter")
                    ),
                }
            }
            UserInputValue::Both(error, input) => {
                match parse_number_binding_value(&input) {
                    Some(value) => ViperBindingValueInput::Both(error, Some(value)),
                    None => ViperBindingValueInput::Left(error),
                }
            }
        };

        setter.set(value);
    });

    let validator = move |input: String| {
        let trimmed_input = input.trim();

        if trimmed_input.is_empty() {
            return UserInputValue::Both(String::from("Please enter a value."), input);
        }

        match trimmed_input.parse::<i64>() {
            Ok(number) => {
                match parameter_descriptor.validate_number_parameter(number) {
                    Ok(_) => UserInputValue::Right(input),
                    Err(error) => {
                        match error.kind {
                            InvalidNumberParameterValueErrorKind::TooSmall { expected, actual: _ } => {
                                UserInputValue::Both(
                                    format!("The number parameter must be at least {expected}."),
                                    input,
                                )
                            }
                            InvalidNumberParameterValueErrorKind::TooBig { expected, actual: _ } => {
                                UserInputValue::Both(
                                    format!("The number parameter must be at most {expected}."),
                                    input,
                                )
                            }
                            InvalidNumberParameterValueErrorKind::InvalidType { expected, actual } => {
                                UserInputValue::Both(
                                    format!("The parameter must be a {expected} (actual: {actual})."),
                                    input,
                                )
                            }
                        }
                    }
                }
            }

            Err(_) => UserInputValue::Both(
                String::from("Invalid parameter type, expected a number"),
                input,
            ),
        }
    };

    let placeholder = {
        match default_value {
            Some(default) => format!("Number Value (Default: {default})"),
            None => String::from("Number Value")
        }
    };

    view! {
        <UserInput
            getter
            setter=input_setter
            validator
            label=display_name.unwrap_or_else(|| name)
            placeholder
            description
            input_type=InputType::Number
            use_default_value
        />
    }
}

fn parse_number_binding_value(input: &str) -> Option<ViperBindingValue> {
    input
        .trim()
        .parse::<i64>()
        .ok()
        .map(ViperBindingValue::NumberValue)
}
