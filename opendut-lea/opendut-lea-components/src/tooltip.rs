use std::sync::Arc;
use leptos::prelude::*;

#[derive(Default, Clone, Copy)]
pub enum TooltipDirection {
    #[default]
    Left,
    Right,
    Up,
    UpRight,
}

impl TooltipDirection {
    fn as_class(&self) -> &'static str {
        match self {
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

    view! {
        <div class=format!("tooltip {}", direction.with(TooltipDirection::as_class))>
            <div class="tooltip-trigger">
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
