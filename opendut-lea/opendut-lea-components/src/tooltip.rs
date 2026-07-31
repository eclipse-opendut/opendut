use leptos::prelude::*;

#[derive(Default)]
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

#[component]
pub fn Tooltip(
    #[prop(into)] text: Signal<String>,
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
                        { text }
                    </div>
                </div>
            </div>
        </div>
    }
}
