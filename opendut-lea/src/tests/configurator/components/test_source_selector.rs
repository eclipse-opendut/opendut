use leptos::prelude::*;
use opendut_lea_components::{SelectionOption, UserSelect};
use crate::app::use_app_globals;
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn TestSourceSelector(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let globals = use_app_globals();

    let registered_sources = {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let mut carl = carl.clone();

            async move {
                carl.viper.list_viper_source_descriptors().await
                    .expect("Failed to request the list of sources")
            }
        })
    };

    let options = Signal::derive(move || {
        if let Some(mut sources) = registered_sources.get() {
            sources.sort_by(|source_a, source_b| {
                    source_a.name.value().to_lowercase()
                        .cmp(&source_b.name.value().to_lowercase())
                });

            sources.iter().map(|source| {
                SelectionOption {
                    display_name: source.name.to_string(),
                    value: source.id.to_string(),
                }
            })
            .collect::<Vec<_>>()
        } else {
            Vec::new()
        }
    });

    let (getter, setter) = create_slice(test_configuration,
        |config| {
            Clone::clone(&config.source)
        },
        |config, input| {
            config.source = input;
        }
    );

    // let validator = |input: String| {
    //     if input.trim().is_empty() {
    //         UserInputValue::Both(String::from("Enter a test source"), input)
    //     } else {
    //         UserInputValue::Right(input)
    //     }
    // };

    view! {
        <UserSelect
            options
            initial_option="Select a test source"
            getter=getter
            setter=setter
            label="Test Source"
        />
    }
}
