use std::ops::Not;

use leptos::prelude::*;
use crate::components::toast::{Toast, ToastContent, ToastKind};

#[component]
pub fn Notification<OnRemove>(
    toast: RwSignal<Toast>,
    on_remove: OnRemove,
) -> impl IntoView
    where
        OnRemove: Fn() + 'static,
{
    let classes = move || move || toast.with(|toast| {
        let color = match toast.kind {
            ToastKind::Info => "is-info",
            ToastKind::Success => "is-success",
            ToastKind::Warning => "is-warning",
            ToastKind::Error => "is-danger",
        };
        format!("dut-toast {color} is-clickable")
    });

    let content = move || toast.with(|toast| {
        match toast.content {
            ToastContent::Simple { ref text } => {
                let text = Clone::clone(text);
                view! { <p>{ text }</p> }
            }
        }
    });

    let life_gauge_percentage = create_read_slice(
        toast,
        |toast| {
            let max: u128 = toast.max_ticks.into();
            let current: u128 = toast.remaining_ticks.get().into();
            (current as f32 / max as f32) * 100_f32
        },
    );

    let life_gauge_value_style = move || life_gauge_percentage.with(|value| {
        format!("width: {value}%;")
    });

    view! {
        <div
            class=classes
            on:click=move |event| {
                event.stop_propagation();
                toast.with_untracked(|toast| {
                    let keep = toast.keep.get_untracked();
                    if keep.not() {
                        toast.keep.set(true);
                    }
                });
            }
        >
            <button
                class="delete"
                on:click=move |event| {
                    event.stop_propagation();
                    on_remove()
                }
            ></button>
            <div class="dut-toast-content">
                { content }
            </div>
            <div class="dut-toast-life-gauge"><div style=life_gauge_value_style></div></div>
        </div>
    }
}
