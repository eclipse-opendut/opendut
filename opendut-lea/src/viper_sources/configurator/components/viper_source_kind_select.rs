use leptos::prelude::*;
use opendut_lea_components::{SelectionOption, UserSelect};
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn ViperSourceKindSelect(user_source_descriptor: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(user_source_descriptor,
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


    view! {
        <UserSelect
            options
            initial_option="Select source kind"
            getter
            setter
            label="Source Kind"
        />
    }
}
