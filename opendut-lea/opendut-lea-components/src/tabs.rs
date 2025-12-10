use leptos::prelude::*;
use crate::FontAwesomeIcon;

#[derive(Clone)]
pub struct Tab {
    pub title: String,
    pub href: String,
    pub is_error: Option<Signal<bool>>,
    pub is_hidden: Option<Signal<bool>>,
}

impl Tab {
    pub fn from_title_and_href(title: String, href: String) -> Self {
        Self {
            title,
            href,
            is_error: None,
            is_hidden: None,
        }
    }

    pub fn with_is_error(mut self, is_error: Signal<bool>) -> Self {
        self.is_error = Some(is_error);
        self
    }

    pub fn with_is_hidden(mut self, is_hidden: Signal<bool>) -> Self {
        self.is_hidden = Some(is_hidden);
        self
    }
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
                        let is_hidden = move || tab.is_hidden.get();

                        view! {
                            <li class=("is-active", is_active) class=move || ("is-hidden", is_hidden().unwrap_or_default())>
                                <a href=tab.href>
                                    <div class="icon-text">
                                        <span class="icon has-text-danger" class:is-hidden=move || !is_error().unwrap_or_default()>
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
