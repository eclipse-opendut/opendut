use leptos::prelude::*;
use opendut_lea_components::InfoMessage;
use crate::viper_tests::configurator::tabs::source::source_selector::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod source_selector;

#[component]
pub fn SourceTab(user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>) -> impl IntoView {

    view! {
        <InfoMessage
            text="" // TODO: description (Why VIPER source in VIPER test?)
        />
        <ViperTestSourceSelector user_test_run_descriptor />
    }
}
