use leptos::prelude::*;
use crate::{ButtonColor, ButtonState};

#[component]
pub fn SimpleButton<A>(
    #[prop(into)] text: Signal<String>,
    #[prop(into)] color: Signal<ButtonColor>,
    #[prop(into)] state: Signal<ButtonState>,
    on_action: A,
) -> impl IntoView
where A: Fn() + 'static {

    let aria_label = Clone::clone(&text);

    let button_class = move || {
        format!("button {} {}",
            color.with(ButtonColor::as_class),
            state.with(ButtonState::as_class),
        )
    };

    view! {
        <button
            class=button_class
            disabled=move || matches!(state.get(), ButtonState::Disabled)
            aria-label=aria_label
            on:click=move |_| on_action()
        >
            <span>
                { text }
            </span>
        </button>
    }
}
