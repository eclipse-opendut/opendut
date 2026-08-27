use std::sync::Arc;
use leptos::prelude::*;
use crate::tooltip::{Tooltip, TooltipDirection};

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
pub fn Health(
    state: Signal<State>,
    #[prop(optional)] tooltip_direction: TooltipDirection,
) -> impl IntoView {

    let health_class = move || state.with(|state| {
        match state.kind {
            StateKind::Unknown => "health-light",
            StateKind::Red => "health-light red",
            StateKind::Yellow => "health-light yellow",
            StateKind::Green => "health-light green",
        }
    });

    let tooltip_text = Signal::derive(move || state.with(|state| {
        Clone::clone(&state.text)
    }));
    
    let tooltip_content = Arc::new(move || {
        view! {
            <p> { tooltip_text } </p>

        }.into_any()
    });

    view! {
        <div class="is-flex is-justify-content-center">
            <Tooltip text=tooltip_content direction=tooltip_direction>
                <div class=health_class />
            </Tooltip>
        </div>
    }
}
