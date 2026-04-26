use leptos::prelude::*;
use crate::viper_tests::configurator::components::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestConfiguration;

#[component]
pub fn SourceTab(viper_test_configuration: RwSignal<UserViperTestConfiguration>) -> impl IntoView {

    view! {
        <ViperTestSourceSelector viper_test_configuration />
    }
}
