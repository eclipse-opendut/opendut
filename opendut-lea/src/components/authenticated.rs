use std::ops::Not;
use gloo_net::http;
use leptos::either::Either;
use leptos::prelude::*;
use leptos_oidc::Auth;
use tracing::info;
use opendut_carl_api::carl::wasm::CarlClient;
use crate::app::{use_app_globals, AppConfig, AppGlobals, AppGlobalsError};

#[must_use]
#[component(transparent)]
pub fn Initialized(
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let globals: LocalResource<Result<AppGlobals, AppGlobalsError>> = LocalResource::new(move || async {
        let config = http::Request::get("/api/lea/config")
            .send().await
            .map_err(|cause| AppGlobalsError { message: format!("Could not fetch configuration:\n  {cause}")})?
            .json::<AppConfig>().await
            .map_err(|cause| AppGlobalsError { message: format!("Could not parse configuration:\n  {cause}")})?;

        info!("Configuration: {config:?}");

        match config.auth_parameters {
            Some(ref auth_parameters) => {
                info!("Auth parameters: {auth_parameters:?}");
                let auth = Auth::init(auth_parameters.clone()).await;
                let client = CarlClient::create(Clone::clone(&config.carl_url), Some(auth.clone())).await
                    .expect("Failed to create CARL client");

                Ok(AppGlobals {
                    config,
                    client,
                    auth: Some(auth),
                })
            },
            None => {
                let client = CarlClient::create(Clone::clone(&config.carl_url), None).await
                    .expect("Failed to create CARL client");
                Ok(AppGlobals {
                    config,
                    client,
                    auth: None,
                })
            }
        }
    });

    let children = StoredValue::new(children);

    view! {
        <Suspense
            fallback=|| view! { <FallbackMessage message="Page is loading..."/> }
        >
            {move || Suspend::new(async move {
                let app_globals_result = globals.await;

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
                        // LeaError::ConfigurationError.navigate_to(); //TODO
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
pub fn InitializedAndAuthenticated(
    children: ChildrenFn,
    #[prop(optional)] _groups: Vec<String>,
    #[prop(optional)] _roles: Vec<String>,
    #[prop(optional, default = true)] authentication_required: bool,
) -> impl IntoView {

    let children = StoredValue::new(children);

    match use_app_globals().auth {
        None => {
            children.read_value()().into_view().into_any()
        }
        Some(auth) => {

            let is_authenticated =  move || { auth.authenticated() };

            // Show component if context is initialized and either the user is authenticated or no authentication is needed.
            let show_component = move || {
                is_authenticated() || authentication_required.not()
            };

            view! {
                <Show
                    when=show_component
                    fallback=|| view! { <FallbackMessage message="You are currently not logged in."/> }
                >
                    {children.read_value()()}
                </Show>
            }.into_any()

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
