use leptos::prelude::*;
use leptos::ev::MouseEvent;

use crate::tooltip::Tooltip;

pub struct State {
    pub kind: StateKind,
    pub text: String,
}

#[allow(dead_code)]
pub enum StateKind {
    Unknown,
    Red,
    Yellow,
    Green,
}

#[component]
pub fn Health(state: Signal<State>) -> impl IntoView {

    let (tooltip_visible, set_tooltip_visible) = signal(false);

    let classes = move || state.with(|state| {
        match state.kind {
            StateKind::Unknown => "health-light",
            StateKind::Red => "health-light red",
            StateKind::Yellow => "health-light yellow",
            StateKind::Green => "health-light green",
        }
    });

    let tool_tip_text = move || state.with(|state| {
        Clone::clone(&state.text)
    });

    view! {
        <div class="is-flex is-justify-content-center">
            <div class=classes
                on:click=move |_| {
                    set_tooltip_visible.set(true);
                }
                on:mouseenter=move |_: MouseEvent| {
                    set_tooltip_visible.set(true);
                }
                on:mouseleave=move |_: MouseEvent| {
                    set_tooltip_visible.set(false)
                }
            >
            </div>
            <div class="height-0">
                <Tooltip visible=tooltip_visible>
                    <span>{ tool_tip_text }</span>
                </Tooltip>
            </div>
        </div>
    }
}
