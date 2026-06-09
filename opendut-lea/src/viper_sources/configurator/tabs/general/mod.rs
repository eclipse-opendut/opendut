use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::viper_sources::configurator::components::ViperSourceKindSelect;
use crate::viper_sources::configurator::tabs::general::name_input::ViperSourceNameInput;
use crate::viper_sources::configurator::tabs::general::url_input::ViperSourceUrlInput;
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

pub mod name_input;
pub mod url_input;

#[component]
pub fn GeneralTab(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let source_id = Signal::derive(move || viper_source_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="ID"
                value=source_id
            />
            <ViperSourceNameInput
                viper_source_configuration
            />
            <ViperSourceUrlInput
                viper_source_configuration
            />
            <ViperSourceKindSelect
                viper_source_configuration
            />
        </div>
    }
}
