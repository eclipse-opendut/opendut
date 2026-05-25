use leptos::prelude::*;

use crate::components::{UserInput, UserInputValue};
use crate::secrets::configurator::types::UserSecretConfiguration;

#[component]
pub fn SecretValueInput(secret_configuration: RwSignal<UserSecretConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(secret_configuration,
        |config| {
            Clone::clone(&config.value)
        },
        |config, input| {
            config.value = input;
        }
    );

    let validator = |input: String| {
        if input.is_empty() {
            UserInputValue::Both("A secret value must not be empty.".to_string(), input)
        } else {
            UserInputValue::Right(input)
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="Secret Value (Token)"
            placeholder="ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
            validator
        />
    }
}
