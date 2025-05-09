use std::ops::Not;
use leptos::prelude::*;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};

#[component]
pub fn CollapseButton(
    collapsed: RwSignal<bool>,
    #[prop(into)] label: Signal<String>,
) -> impl IntoView {
    let collapse_button_icon = Signal::derive(move ||
        if collapsed.get() {
            FontAwesomeIcon::ChevronDown
        } else {
            FontAwesomeIcon::ChevronUp
        }
    );

    view! {
        <IconButton
            icon=collapse_button_icon
            color=ButtonColor::White
            size=ButtonSize::Small
            state=ButtonState::Enabled
            label
            on_action=move || collapsed.update(|collapsed| *collapsed = collapsed.not())
        />
    }
}
