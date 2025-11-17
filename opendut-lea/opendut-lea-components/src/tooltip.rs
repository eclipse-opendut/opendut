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
    #[prop(default=TooltipDirection::Left)] direction: TooltipDirection,
    #[prop(optional, into)] is_hidden: Option<Signal<bool>>,
    children: Children
) -> impl IntoView {

    let is_hidden = move || is_hidden.as_ref().map(|is_hidden| is_hidden.get()).unwrap_or(false);

    view! {
        <div class=format!("tooltip {}", direction.as_class())>
            <div class="tooltip-trigger">
                { children() }
            </div>
            <div class="tooltip-container" style=move || if is_hidden() { "display: none" } else { "" }>
                <div class="tooltip-content">
                    <div class="tooltip-item">
                        { text }
                    </div>
                </div>
            </div>
        </div>
    }
}
