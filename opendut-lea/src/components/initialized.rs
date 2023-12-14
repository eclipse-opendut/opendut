use leptos::{ChildrenFn, component, IntoView, Show, SignalGet, view};

use crate::app::use_app_globals;
use crate::routing::{navigate_to, WellKnownRoutes};

#[component(transparent)]
pub fn Initialized(
    children: ChildrenFn,
) -> impl IntoView {

    let globals = use_app_globals();
    let successfully_initialized = move || {
        let is_initialized = !globals.loading().get();
        let is_successful = globals.get()
            .map(|result| result.is_ok())
            .unwrap_or(false);
        is_initialized && is_successful
    };
    let fallback = move || {
        match globals.get() {
            Some(Err(error)) => {
                navigate_to(WellKnownRoutes::ErrorPage {
                    title: String::from("Configuration Error"),
                    text: error.message,
                    details: None,
                });
                view! { <p></p> }
            }
            _ => {
                view! {
                    <p><i class="fa-solid fa-circle-notch fa-spin"></i></p>
                }
            }
        }
    };

    view! {
        <Show
            when=successfully_initialized
            fallback=fallback
            children=children
        >
        </Show>
    }
}
