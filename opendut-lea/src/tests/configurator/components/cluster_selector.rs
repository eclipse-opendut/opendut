use leptos::prelude::*;

use crate::components::{UserInput, UserInputValue};
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn ClusterSelector(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(test_configuration,
        |config| {
            Clone::clone(&config.cluster)
        },
        |config, input| {
            config.cluster = input;
        }
    );

    let validator = |input: String| {
        if input.trim().is_empty() {
            UserInputValue::Both(String::from("Enter a cluster ID"), input)
        } else {
            UserInputValue::Right(input)
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Cluster ID"
            placeholder="00000000-0000-0000-0000-000000000000"
            validator=validator
        />
    }
}
