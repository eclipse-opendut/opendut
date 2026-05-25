use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::viper_sources::configurator::components::{ViperSourceKindSelect, ViperSourceNameInput, ViperSourceSecretSelect, ViperSourceUrlInput};
use crate::viper_sources::configurator::types::UserViperSourceConfiguration;

#[component]
pub fn GeneralTab(viper_source_configuration: RwSignal<UserViperSourceConfiguration>) -> impl IntoView {

    let source_id = Signal::derive(move || viper_source_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Source ID"
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
            <ViperSourceSecretSelect
                viper_source_configuration
            />
        </div>
    }
}
