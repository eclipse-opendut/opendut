use std::ops::Not;
use leptos::either::Either;
use leptos::prelude::*;
use opendut_auth::public::Authentication;
use crate::{app::{use_app_globals, AppGlobals, AppGlobalsError}, components::LoadingSpinner};

#[must_use]
#[component(transparent)]
pub fn Initialized(
    app_globals: AppGlobalsResource,
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let children = StoredValue::new(children);

    view! {
        <Suspense
            fallback=LoadingSpinner
        >
            {move || Suspend::new(async move {
                let app_globals_result = app_globals.await;

                match app_globals_result {
                    Ok(app_globals) => {
                        provide_context(app_globals);

                        Either::Right(view! {
                            <InitializedAndAuthenticated
                                authentication_required=authentication_required
                            >
                                { children.read_value()() }
                            </InitializedAndAuthenticated>
                        })
                    }
                    Err(error) => {
                        tracing::error!("Error while constructing app globals: {}", error);
                        Either::Left(
                            view! { <FallbackMessage message=""/> }
                        )
                    }
                }
            })}
        </Suspense>
    }
}

#[component]
fn InitializedAndAuthenticated(
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let children = StoredValue::new(children);

    match use_app_globals().auth {
        Authentication::Disabled => {
            Either::Left(children.read_value()().into_any())
        }
        Authentication::Enabled(auth) => {

            let is_authenticated =  move || { auth.get().is_authenticated() };

            // Show component if context is initialized and either the user is authenticated or no authentication is needed.
            let show_component = move || {
                is_authenticated() || authentication_required.not()
            };

            Either::Right(view! {
                <Show
                    when=show_component
                    fallback=|| view! { <FallbackMessage message="You are currently not logged in."/> }
                >
                    {children.read_value()()}
                </Show>
            })
        }
    }
}

#[component]
fn FallbackMessage(message: &'static str) -> impl IntoView {
    view! {
        <p>
            <div class="columns is-full">
                <div class="column">
                    <h1 class="title is-5 has-text-centered">{ message }</h1>
                </div>
            </div>
        </p>
    }
}

pub type AppGlobalsResource = LocalResource<Result<AppGlobals, AppGlobalsError>>;
