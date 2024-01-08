use leptos::*;

use crate::components::{ButtonColor, ButtonState, FontAwesomeIcon};

#[component]
pub fn SimpleButton<A>(
    #[prop(into)] text: MaybeSignal<String>,
    #[prop(into)] color: MaybeSignal<ButtonColor>,
    #[prop(into)] state: MaybeSignal<ButtonState>,
    on_action: A,
) -> impl IntoView
where A: Fn() + 'static {

    let aria_label = Clone::clone(&text);

    view! {
        <button
            class=move || format!("button {}", color.with(ButtonColor::as_class))
            class=("is-hidden", move || matches!(state.get(), ButtonState::Hidden))
            disabled=move || matches!(state.get(), ButtonState::Disabled | ButtonState::Loading)
            aria-label=aria_label
            on:click=move |_| on_action()
        >
            <span
                style=move || if ButtonState::Loading == state.get() { "visibility: hidden;" } else { "" }
            >
                { text }
            </span>
            <i
                class=move || format!("{} fa-spin", FontAwesomeIcon::CircleNotch.as_class())
                class=("is-hidden", move || ButtonState::Loading != state.get())
                style="position: absolute;"
            />
        </button>
    }
}
