use leptos::prelude::*;
use opendut_lea_components::{SelectionOption, UserSelect};
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceKindSelect(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(viper_source_configuration,
        |config| {
            Clone::clone(&config.kind)
        },
        |config, input| {
            config.kind = input;
        }
    );

    let options = Signal::derive(move || {
        vec![
            SelectionOption { display_name: String::from("HTTP"), value: String::from("HTTP") },
            SelectionOption { display_name: String::from("Git"), value: String::from("Git") },
        ]
    });

    let initial_option = Signal::derive(|| String::from("Select source kind"));

    view! {
        <UserSelect
            options
            initial_option
            getter=getter
            setter=setter
            label="Source Kind"
        />
    }
}
