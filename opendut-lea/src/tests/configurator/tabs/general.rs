use leptos::prelude::*;
use opendut_lea_components::ReadOnlyInput;
use crate::tests::configurator::components::{TestNameInput, TestSourceSelector, TestSuiteSelector};
use crate::tests::configurator::types::UserTestConfiguration;

#[component]
pub fn GeneralTab(test_configuration: RwSignal<UserTestConfiguration>) -> impl IntoView {

    let test_id = Signal::derive(move || test_configuration.get().id.to_string());

    view! {
        <div>
            <ReadOnlyInput
                label="Test ID"
                value=test_id
            />
            <TestNameInput
                test_configuration
            />
            <TestSourceSelector
                test_configuration
            />
            <TestSuiteSelector
                test_configuration
            />
        </div>
    }
}
