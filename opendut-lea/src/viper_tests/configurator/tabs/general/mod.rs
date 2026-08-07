use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::viper_tests::configurator::tabs::general::name_input::ViperTestNameInput;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod name_input;

#[component]
pub fn GeneralTab(user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    let test_id = Signal::derive(move || user_test_run_descriptor.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="ID"
                value=test_id
            />
            <ViperTestNameInput
                user_test_run_descriptor
            />
        </div>
    }
}
