use leptos::prelude::*;
use opendut_lea_components::{InputType, UserInput, UserInputValue};
use opendut_model::viper::ViperBindingValue;
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn NumberParameterInput(
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    default: Option<i64>,
    min: i64,
    max: i64,
) -> impl IntoView {

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
            Ok(number) if number < min => UserInputValue::Both(
                format!("The number parameter must be at least {min}."),
                input,
            ),
            Ok(number) if number > max => UserInputValue::Both(
                format!("The number parameter must be at most {max}."),
                input,
            ),
            Ok(_) => UserInputValue::Right(input),
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
