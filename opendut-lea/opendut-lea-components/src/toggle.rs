use std::ops::Not;
use leptos::prelude::*;
use crate::FontAwesomeIcon;

#[derive(Clone, Debug, Default)]
pub enum ToggleState {
    #[default] Enabled,
    Disabled,
    Loading,
}

#[component]
pub fn Toggle<F>(
    #[prop(optional, into)] text: Option<Signal<String>>,
    #[prop(optional, into)] state: Signal<ToggleState>,
    is_active: Signal<bool>,
    on_action: F,
) -> impl IntoView
where F: Fn() + 'static {

    let is_disabled = Signal::derive(move || matches!(state.get(), ToggleState::Enabled).not());
    let is_loading = move || matches!(state.get(), ToggleState::Loading);

    view! {
        <div
            class="is-flex is-align-items-center is-justify-content-center"
            on:click=move |event| event.stop_propagation()
        >
            <label
                class="dut-toggle"
                class=("active", move || is_active.get())
                class=("toggle-disabled", is_disabled)
                on:click=move |_| {
                    if !is_disabled.get() {
                        on_action()
                    }
                }
            >
                <span class="bubble">
                    { move ||
                        if is_loading() {
                            view! {
                                <i class=FontAwesomeIcon::CircleFadeAnimation.as_class() />
                            }
                        } else {
                            view! {
                                <i class=FontAwesomeIcon::Circle.as_class() />
                            }
                        }
                    }
                </span>
            </label>
            {
                text.map(|text| {
                    view! {
                        <span class="pl-2">{ text }</span>
                    }
                })
            }
        </div>
    }
}
