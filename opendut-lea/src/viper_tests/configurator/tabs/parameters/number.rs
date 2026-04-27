use leptos::prelude::*;
use opendut_lea_components::{InputType, UserInput, UserInputValue};
use opendut_model::viper::{InvalidNumberParameterValueErrorKind, ViperBindingValue, ViperParameterDescriptor, ViperParameterInfo};
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn NumberParameterInput(
    parameter_descriptor: ViperParameterDescriptor,
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
) -> impl IntoView {

    let name = parameter_descriptor.name().to_string();
    let ViperParameterInfo { display_name, description } = parameter_descriptor.info().to_owned();

    let getter = Signal::derive(move || {
        if let Some(getter_value) = getter.get() {
            match getter_value {
                ViperBindingValueInput::Left(error) => UserInputValue::Left(error),
                ViperBindingValueInput::Right(ViperBindingValue::NumberValue(number)) => UserInputValue::Right(number.to_string()),
                ViperBindingValueInput::Both(err, ViperBindingValue::NumberValue(number)) => UserInputValue::Both(err, number.to_string()),

                ViperBindingValueInput::Right(_) => UserInputValue::Left(String::from("Invalid parameter type, expected a number parameter")),
                ViperBindingValueInput::Both(error, _) => UserInputValue::Left(error),
            }
        } else {
            UserInputValue::Right(String::new())
        }
    });

    let setter = SignalSetter::map(move |value: UserInputValue| {
        let value = match value {
            UserInputValue::Left(error) => {
                ViperBindingValueInput::Left(error)
            }
            UserInputValue::Right(input) => {
                if let Ok(parsed_value) = input.parse::<i64>() {
                    let value = ViperBindingValue::NumberValue(parsed_value);
                    ViperBindingValueInput::Right(value)
                } else {
                    ViperBindingValueInput::Left(String::from("Invalid parameter type, expected a number parameter"))
                }
            }
            UserInputValue::Both(error, input) => {
                if let Ok(parsed_value) = input.parse::<i64>() {
                    let value = ViperBindingValue::NumberValue(parsed_value);
                    ViperBindingValueInput::Both(error, value)
                } else {
                    ViperBindingValueInput::Left(String::from("Invalid parameter type, expected a number parameter"))
                    // Fixme: why is this triggered when the input field is empty?
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

    view! {
        <UserInput
            getter
            setter
            validator
            label=display_name.unwrap_or_else(|| name)
            placeholder="Number Parameter"
            description
            input_type=InputType::Number
        />
    }
}
