use leptos::*;

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon};

#[component]
pub fn IconButton<A>(
    #[prop(into)] icon: MaybeSignal<FontAwesomeIcon>,
    #[prop(into)] color: MaybeSignal<ButtonColor>,
    #[prop(into)] size: MaybeSignal<ButtonSize>,
    #[prop(into)] state: MaybeSignal<ButtonState>,
    #[prop(into)] label: MaybeSignal<String>,
    #[prop(into, default = MaybeSignal::from(false))] show_label: MaybeSignal<bool>,
    on_action: A,
) -> impl IntoView
where A: Fn() + 'static {

    let cloned_label = label.clone();

    view! {
        <button
            class=move || format!("button {} {}", color.with(ButtonColor::as_class), size.with(ButtonSize::as_class))
            class=("is-hidden", move || matches!(state.get(), ButtonState::Hidden))
            disabled=move || matches!(state.get(), ButtonState::Disabled | ButtonState::Loading)
            aria-label=move || label.get()
            on:click=move |event| {
                event.stop_propagation();
                on_action();
            }
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
            {
                show_label.get().then(|| {
                    view! {
                        <span>{ cloned_label.get() }</span>
                    }
                })
            }
        </button>
    }
}
