use leptos::prelude::*;
use opendut_lea_components::{UserInput, UserInputValue};
use opendut_model::viper::ViperBindingValue;
use crate::viper_tests::configurator::types::ViperBindingValueInput;

#[component]
pub fn TextParameterInput(
    getter: Signal<Option<ViperBindingValueInput>>,
    setter: SignalSetter<ViperBindingValueInput>,
    name: String,
    display_name: Option<String>,
    description: Option<String>,
    default: Option<String>,
    max: u32,
) -> impl IntoView {

    let getter = Signal::derive(move || {
        let getter_value = getter.get()
            .unwrap_or(ViperBindingValueInput::Right(ViperBindingValue::TextValue(String::new())));

        match getter_value {
            ViperBindingValueInput::Left(error) => UserInputValue::Left(error),
            ViperBindingValueInput::Right(ViperBindingValue::TextValue(text)) => UserInputValue::Right(text),
            ViperBindingValueInput::Both(err, ViperBindingValue::TextValue(text)) => UserInputValue::Both(err, text),

            ViperBindingValueInput::Right(_) => UserInputValue::Left(String::from("Invalid parameter type, expected a text parameter")),
            ViperBindingValueInput::Both(error, _) => UserInputValue::Left(error),
        }
    });

    let setter = SignalSetter::map(move |value: UserInputValue| {
        let value = match value {
            UserInputValue::Left(error) => {
                ViperBindingValueInput::Left(error)
            }
            UserInputValue::Right(value) => {
                let value = ViperBindingValue::TextValue(value);
                ViperBindingValueInput::Right(value)
            }
            UserInputValue::Both(error, value) => {
                let value = ViperBindingValue::TextValue(value);
                ViperBindingValueInput::Both(error, value)
            }
        };

        setter.set(value);
    });

    let validator = move |input: String| {
        if input.trim().is_empty() {
            UserInputValue::Both(
                "Please enter a value.".to_string(),
                input,
            )
        } else if input.len() > max as usize {
            UserInputValue::Both(
                format!("The text parameter must be at most {max} characters long."),
                input,
            )
        } else {
            UserInputValue::Right(input)
        }
    };


    view! {
        <UserInput
            getter
            setter
            validator
            label=display_name.unwrap_or_else(|| name)
            placeholder="Text Parameter"
        />
    }
}
