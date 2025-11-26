use leptos::prelude::*;

mod tabs;
mod types;
mod components;

#[component(transparent)]
pub fn TestConfigurator() -> impl IntoView {

    view! {
        <div>
            Test Configurator
        </div>
    }
}
