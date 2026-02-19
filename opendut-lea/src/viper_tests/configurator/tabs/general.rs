use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::viper_tests::configurator::components::ViperTestNameInput;
use crate::viper_tests::configurator::types::UserViperTestConfiguration;


#[component]
pub fn GeneralTab(viper_test_configuration: RwSignal<UserViperTestConfiguration>) -> impl IntoView {

    let test_id = Signal::derive(move || viper_test_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Test ID"
                value=test_id
            />
            <ViperTestNameInput
                viper_test_configuration
            />
        </div>
    }
}
