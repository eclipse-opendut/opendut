use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::viper_tests::configurator::tabs::general::name_input::ViperTestNameInput;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod name_input;

#[component]
pub fn GeneralTab(viper_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let test_id = Signal::derive(move || viper_test_run_descriptor.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Test ID"
                value=test_id
            />
            <ViperTestNameInput
                viper_test_run_descriptor
            />
        </div>
    }
}
