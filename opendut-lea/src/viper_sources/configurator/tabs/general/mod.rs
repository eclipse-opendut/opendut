use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::viper_sources::configurator::components::ViperSourceKindSelect;
use crate::viper_sources::configurator::tabs::general::name_input::ViperSourceNameInput;
use crate::viper_sources::configurator::tabs::general::url_input::ViperSourceUrlInput;
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

pub mod name_input;
pub mod url_input;

#[component]
pub fn GeneralTab(user_source_descriptor: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let source_id = Signal::derive(move || user_source_descriptor.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="ID"
                value=source_id
            />
            <ViperSourceNameInput
                user_source_descriptor
            />
            <ViperSourceUrlInput
                user_source_descriptor
            />
            <ViperSourceKindSelect
                user_source_descriptor
            />
        </div>
    }
}
