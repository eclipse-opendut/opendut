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

    let classes = move || state.with(|state| {
        match state.kind {
            StateKind::Unknown => "health-light",
            StateKind::Red => "health-light red",
            StateKind::Yellow => "health-light yellow",
            StateKind::Green => "health-light green",
        }
    });

    let tool_tip_text = Signal::derive(move || state.with(|state| {
        Clone::clone(&state.text)
    }));

    view! {
        <div class="is-flex is-justify-content-center">
            <Tooltip text=tool_tip_text>
                <div class=classes />
            </Tooltip>
        </div>
    }
}
