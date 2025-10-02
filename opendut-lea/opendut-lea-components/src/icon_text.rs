use leptos::prelude::*;
use crate::FontAwesomeIcon;

#[component]
pub fn IconText(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] text: Signal<String>,
    #[prop(into, optional)] href: Option<Signal<String>>,
) -> impl IntoView {

    let displayed_text = match href {
        Some(link) => {
            view! {
                <a href=link.get() class="dut-text-nowrap">
                    { text }
                </a>
            }.into_any()
        }
        None => {
            view! {
                <span class="dut-text-nowrap">
                    { text }
                </span>
            }.into_any()
        }
    };

    view! {
        <div class="is-flex is-align-content-center">
            <span class="icon pr-2">
                <i class=icon.read().as_class() />
            </span>
            { displayed_text }
        </div>
    }
}