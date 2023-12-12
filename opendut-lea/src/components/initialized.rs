use leptos::{ChildrenFn, component, IntoView, Show, SignalGet, view};

use crate::app::use_app_globals;

#[component(transparent)]
pub fn Initialized(
    children: ChildrenFn,
) -> impl IntoView {

    let globals = use_app_globals();
    let initialized = move || !globals.loading().get();

    view! {
        <Show
            when=initialized
            fallback=|| view! {
                <i class="fa-solid fa-circle-notch fa-spin"></i>
            }
            children=children
        >
        </Show>
    }
}
