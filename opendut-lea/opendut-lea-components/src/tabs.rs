use leptos::prelude::*;
use crate::FontAwesomeIcon;

#[derive(Clone)]
pub struct Tab {
    pub title: String,
    pub href: String,
    pub is_error: Signal<bool>,
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
                        let is_error = move || tab.is_error.get();

                        view! {
                            <li class=("is-active", is_active)>
                                <a href=tab.href>
                                    <div class="icon-text">
                                        <span class="icon has-text-danger" class:is-hidden=move || !is_error()>
                                            <i class=FontAwesomeIcon::CircleExclamation.as_class()></i>
                                        </span>
                                        <span>{ tab.title }</span>
                                    </div>
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
