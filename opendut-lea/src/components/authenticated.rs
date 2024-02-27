use std::ops::Not;
use leptos::{ChildrenFn, component, HtmlElement, IntoView, Show, SignalGet, view};
use leptos::html::P;
use crate::app::{ExpectGlobals, use_app_globals};
use crate::routing::{navigate_to, WellKnownRoutes};

#[must_use]
#[component(transparent)]
pub fn Initialized(
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let globals = use_app_globals();
    let successfully_initialized = move || {
        let is_initialized = !globals.loading().get();
        let is_successful = globals.get()
            .map(|result| result.is_ok())
            .unwrap_or(false);
        is_initialized && is_successful
    };

    let fallback = move || { fallback("Configuration Error", "Page is loading.") };

    view! {
        <Show
            fallback=fallback
            when=successfully_initialized
        >
            <InitializedAndAuthenticated
                authentication_required=authentication_required
                children=children.clone()
            >
            </InitializedAndAuthenticated>
        </Show>
    }
}

#[component]
pub fn InitializedAndAuthenticated(
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    match use_app_globals().expect_auth() {
        None => {
            children.into_view()
        }
        Some(auth) => {

            let is_authenticated =  move || { auth.authenticated() };

            // Show component if context is initialized and either the user is authenticated or no authentication is needed.
            let show_component = move || {
                is_authenticated() || authentication_required.not()
            };

            let fallback = move || { fallback("Authentication Error", "You are currently not logged in.") };

            view! {
                <Show
                    when=show_component
                    fallback=fallback
                    children=children
                >
                </Show>
            }

        }
    }
}

fn fallback(error_message: &'static str, fallback_loading_message: &'static str) -> HtmlElement<P> {
    return match use_app_globals().get() {
        Some(Err(error)) => {
            navigate_to(WellKnownRoutes::ErrorPage {
                title: String::from(error_message),
                text: error.message,
                details: None,
            });
            view! { <p></p> }
        }
        _ => {
            view! {
                <p>
                    <div class="columns is-full">
                        <div class="column">
                            <h1 class="title is-5 has-text-centered">{ fallback_loading_message }</h1>
                        </div>
                    </div>
                </p>
            }
        }
    };
}