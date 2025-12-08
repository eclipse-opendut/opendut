use leptos::prelude::*;

// #[derive(Debug, Clone, Copy, Default)]
// pub enum TabState {
//     #[default]
//     Normal,
//     Success,
//     Warning,
//     Error,
// }

#[derive(Clone)]
pub struct Tab {
    pub title: String,
    pub href: String,
    // pub state: TabState,
}

#[component]
pub fn Tabs(
    #[prop(into)] tabs: Signal<Vec<Tab>>,
    #[prop(into)] active_tab: Signal<String>,
    children: Children,
) -> impl IntoView {

    view! {
        <div class="tabs">
            <ul>
                <For
                    each=move || tabs.get()
                    key=|tab| Clone::clone(&tab.title)
                    children=move |tab| {

                        let href = Clone::clone(&tab.href);
                        let is_active = move || href.to_lowercase() == active_tab.get().to_lowercase();

                        view! {
                            <li class=("is-active", is_active)>
                                <a href=tab.href>
                                    { Clone::clone(&tab.title) }
                                </a>
                            </li>
                        }
                    }
                />
            </ul>
        </div>
        <div class="container">
            { children() }
        </div>
    }
}
