use leptos::prelude::*;
use crate::Tag;

#[component]
pub fn TagFromList(items: Vec<String>) -> impl IntoView {
    view! {
        {
            if items.is_empty() {
                view! ("-").into_any()
            } else { ().into_any() }
        }
        <For
            each=move || items.clone()
            key=|item| item.to_string()
            children=move |item| {
                view! {
                    <span class="pr-3">
                        <Tag text=item />
                    </span>
                }
            }
        />
    }
}
