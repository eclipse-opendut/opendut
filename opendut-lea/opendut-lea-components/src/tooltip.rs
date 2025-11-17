use leptos::prelude::*;

pub enum TooltipDirection {
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

#[component]
pub fn Tooltip(
    #[prop(into)] text: Signal<String>,
    #[prop(into, default=Signal::from(TooltipDirection::Left))] direction: Signal<TooltipDirection>,
    #[prop(into, default=Signal::from(false))] is_hidden: Signal<bool>,
    children: Children
) -> impl IntoView {

    view! {
        <div class=format!("tooltip {}", direction.with(TooltipDirection::as_class))>
            <div class="tooltip-trigger">
                { children() }
            </div>
            <div class="tooltip-container" style=move || if is_hidden.get() { "display: none" } else { "" }>
                <div class="tooltip-content">
                    <div class="tooltip-item">
                        { text }
                    </div>
                </div>
            </div>
        </div>
    }
}
