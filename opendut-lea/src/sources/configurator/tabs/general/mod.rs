use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::sources::configurator::components::{SourceNameInput, SourceUrlInput};
use crate::sources::configurator::types::UserSourceConfiguration;

#[component]
pub fn GeneralTab(source_configuration: RwSignal<UserSourceConfiguration>) -> impl IntoView {

    let source_id = Signal::derive(move || source_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Source ID"
                value=source_id
            />
            <SourceNameInput
                source_configuration
            />
            <SourceUrlInput
                source_configuration
            />
        </div>
    }
}
