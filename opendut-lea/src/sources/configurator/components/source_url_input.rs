use leptos::prelude::*;
use url::Url;
use crate::components::{UserInput, UserInputValue};
use crate::sources::configurator::types::UserSourceConfiguration;

#[component]
pub fn SourceUrlInput(source_configuration: RwSignal<UserSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(source_configuration,
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
            Err(_) => { todo!() }
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Source Url"
            placeholder="https://example.com"
            validator=validator
        />
    }
}
