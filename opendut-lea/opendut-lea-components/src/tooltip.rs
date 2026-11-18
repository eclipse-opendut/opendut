use std::sync::Arc;
use leptos::html::Div;
use leptos::prelude::*;
use leptos_use::{use_element_bounding, use_window_size, UseElementBoundingReturn, UseWindowSizeReturn};

#[derive(Default, Clone, Copy)]
pub enum TooltipDirection {
    #[default]
    Auto,
    Left,
    Right,
    Up,
    UpRight,
}

impl TooltipDirection {
    fn as_class(&self) -> &'static str {
        match self {
            TooltipDirection::Auto => "",
            TooltipDirection::Left => "is-left",
            TooltipDirection::Right => "is-right",
            TooltipDirection::Up => "is-up",
            TooltipDirection::UpRight => "is-right is-up",
        }
    }
}

pub type TooltipContent = Arc<dyn Fn() -> AnyView + Send + Sync>;

#[component]
pub fn Tooltip(
    text: TooltipContent,
    #[prop(into, optional)] direction: Signal<TooltipDirection>,
    #[prop(into, default=Signal::from(false))] is_hidden: Signal<bool>,
    children: Children
) -> impl IntoView {
    let trigger = NodeRef::<Div>::new();
    let UseElementBoundingReturn { left, width: trigger_width, update, .. } =
        use_element_bounding(trigger);
    let UseWindowSizeReturn { width: window_width, .. } = use_window_size();

    let direction_class = move || {
        let direction = match direction.get() {
            TooltipDirection::Auto => {
                let trigger_center = left.get() + trigger_width.get() / 2.0;
                if trigger_center > window_width.get() / 2.0 {
                    TooltipDirection::Right
                } else {
                    TooltipDirection::Left
                }
            }
            direction => direction,
        };
        format!("tooltip {}", direction.as_class())
    };

    view! {
        <div class=direction_class>
            <div class="tooltip-trigger" node_ref=trigger on:mouseenter=move |_| update()>
                { children() }
            </div>
            <div class="tooltip-container" class=("is-hidden", move || is_hidden.get())>
                <div class="tooltip-content p-0">
                    <div class="tooltip-item p-3">
                        { move || text() }
                    </div>
                </div>
            </div>
        </div>
    }
}
