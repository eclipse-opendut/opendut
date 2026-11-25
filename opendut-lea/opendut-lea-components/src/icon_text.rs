use leptos::prelude::*;
use crate::FontAwesomeIcon;

#[derive(Default)]
pub enum IconTextDistance {
    #[default]
    Normal,
    Wide
}

#[component]
pub fn IconText(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] text: Signal<String>,
    #[prop(optional)] distance: IconTextDistance,
    #[prop(into, optional)] href: Option<Signal<String>>,
) -> impl IntoView {

    let span_class = format!("icon {}",
        match distance {
            IconTextDistance::Normal => "pr-2",
            IconTextDistance::Wide => "pr-5"
        }
    );

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
            <span class=span_class>
                <i class=icon.read().as_class() />
            </span>
            { displayed_text }
        </div>
    }
}
