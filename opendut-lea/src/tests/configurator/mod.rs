use leptos::prelude::*;
use crate::components::tabs::{Tab, TabIdentifier, TabState, ConfiguratorTabs};
use crate::tests::configurator::tabs::{GeneralTab, ParameterTab};

mod tabs;
mod types;
mod components;

#[component(transparent)]
pub fn TestConfigurator() -> impl IntoView {

    let tabs = vec![
        Tab::new(TabIdentifier::General, String::from("General"), TabState::Normal, || view! {<GeneralTab />}.into_any()),
        Tab::new(TabIdentifier::Parameters, String::from("Parameters"), TabState::Normal, || view! {<ParameterTab />}.into_any()),
    ];

    view! {
        <div>
            <ConfiguratorTabs tabs />
        </div>
    }
}
