use leptos::*;

use crate::components::{ButtonColor, ButtonState, FontAwesomeIcon};

#[component]
pub fn IconButton<A>(
    #[prop(into)] icon: MaybeSignal<FontAwesomeIcon>,
    #[prop(into)] color: MaybeSignal<ButtonColor>,
    #[prop(into)] state: MaybeSignal<ButtonState>,
    #[prop(into)] label: MaybeSignal<String>,
    on_action: A,
) -> impl IntoView
where A: Fn() + 'static {

    view! {
        <button
            class=move || color.with(ButtonColor::as_class)
            class=("is-hidden", move || matches!(state.get(), ButtonState::Hidden))
            disabled=move || matches!(state.get(), ButtonState::Disabled | ButtonState::Loading)
            aria-label=move || label.get()
            on:click=move |_| on_action()
        >
            <span class="icon">
                <i class=move || {
                    if matches!(state.get(), ButtonState::Loading) {
                        format!("{} fa-spin", FontAwesomeIcon::CircleNotch.as_class())
                    }
                    else {
                        String::from(icon.with(FontAwesomeIcon::as_class))
                    }
                }/>
            </span>
        </button>
    }
}
