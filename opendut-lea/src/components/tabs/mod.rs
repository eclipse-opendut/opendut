mod types;
mod util;

use leptos::prelude::*;
pub use crate::components::tabs::types::Tab;
pub use crate::components::tabs::types::TabState;
pub use crate::components::tabs::types::TabIdentifier;
use crate::components::tabs::util::use_active_tab;

#[component]
pub fn ConfiguratorTabs(
    #[prop(into)] tabs: Signal<Vec<Tab>>
) -> impl IntoView {

    let tabs_in_page = Signal::derive(move || {
        let tabs = tabs.get();
        tabs.iter().map(|tab| tab.id).collect::<Vec<_>>()
    });
    let active = use_active_tab::<TabIdentifier>(tabs_in_page);

    view! {
        <div class="tabs">
            <ul>
                <For
                    each=move || tabs.get()
                    key=|tab| tab.id
                    children=move |tab| {
                        let is_active = move || active.get() == tab.id;
                        view! {
                            <li class=("is-active", is_active)>
                                <a href=tab.id.as_str()>
                                    { tab.title }
                                </a>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
        <div class="class">
            { move || {
                let tabs = tabs.get();
                let active_tab_id = active.get();
                let active_tab = tabs.iter().find(|tab| tab.id == active_tab_id);

                active_tab.map(|tab| {
                    tab.render
                })
            }}
        </div>
    }
}
