use std::ops::Not;
use leptos::prelude::*;
use crate::FontAwesomeIcon;

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
        <div
            class="dut-collapse-button"
            role="button"
            aria-label=label
            on:click=move |event| {
                event.stop_propagation();
                collapsed.update(|collapsed| *collapsed = collapsed.not())
            }
        >
            <i class=move || collapse_button_icon.get().as_class()/>
        </div>
    }
}
