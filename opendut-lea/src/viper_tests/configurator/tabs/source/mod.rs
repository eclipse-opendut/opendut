use leptos::prelude::*;
use opendut_lea_components::{ErrorMessage, ErrorText, InfoMessage};
use opendut_model::viper::ViperSourceDescriptor;
use crate::viper_tests::configurator::tabs::source::source_selector::ViperTestSourceSelector;
use crate::viper_tests::configurator::types::UserViperTestRunDescriptor;

pub mod source_selector;

#[component]
pub fn SourceTab(
    user_test_run_descriptor: RwSignal<UserViperTestRunDescriptor>,
    viper_sources: ReadSignal<Result<Vec<ViperSourceDescriptor>, ErrorText>>,
    #[prop(into)] on_retry: Callback<()>,
) -> impl IntoView {

    view! {
        <InfoMessage
            text="Select the Source to load your test code from."
        />
        { move || match viper_sources.get() {
            Ok(viper_sources) => {
                view! {
                    <ViperTestSourceSelector user_test_run_descriptor viper_sources />
                }.into_any()
            }
            Err(error)=> {
                view!(<ErrorMessage text=error on_retry />).into_any()
            }
        }}
    }
}
