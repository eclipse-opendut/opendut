use leptos::prelude::*;

use crate::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon};

#[component]
pub fn IconButton<A>(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] color: Signal<ButtonColor>,
    #[prop(into)] size: Signal<ButtonSize>,
    #[prop(into)] state: Signal<ButtonState>,
    #[prop(into)] label: Signal<String>,
    #[prop(into, default=Signal::from(false))] show_label: Signal<bool>,
    #[prop(into, default=Signal::from(false))] is_outlined: Signal<bool>,
    on_action: A,
) -> impl IntoView
where A: Fn() + 'static {

    let button_class = move || {
        format!("button {} {} {} {}",
            color.with(ButtonColor::as_class),
            state.with(ButtonState::as_class),
            size.with(ButtonSize::as_class),
            if is_outlined.get() { "is-outlined "} else { "" }
        )
    };

    view! {
        <button
            class=button_class
            disabled=move || matches!(state.get(), ButtonState::Disabled | ButtonState::Loading)
            aria-label=move || label.get()
            on:click=move |event| {
                event.stop_propagation();
                on_action();
            }
        >
            <span class="icon">
                <i class=icon.with(FontAwesomeIcon::as_class)/>
            </span>
            {
                show_label.get().then(|| {
                    view! {
                        <span>{ label }</span>
                    }
                })
            }
        </button>
    }
}
