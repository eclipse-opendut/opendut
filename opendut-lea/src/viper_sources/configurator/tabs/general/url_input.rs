use leptos::prelude::*;
use url::Url;
use crate::components::{UserInput, UserInputValue};
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceUrlInput(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_source_configuration,
        |config| {
            Clone::clone(&config.url)
        },
        |config, input| {
            config.url = input;
        }
    );

    let validator = |input: String| {
        match Url::parse(&input) {
            Ok(_) => { UserInputValue::Right(input) }
            Err(_) => { UserInputValue::Both("Enter a valid VIPER source URL.".to_string(), input) }
        }
    };

    view! {
        <UserInput
            getter
            setter
            label="URL"
            placeholder="https://example.com"
            validator
        />
    }
}
