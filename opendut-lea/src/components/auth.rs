use leptos::either::Either;
use leptos::prelude::*;
use leptos_oidc::{LoginLink, LogoutLink};
use opendut_auth::public::Authentication;

use crate::app::use_app_globals;
use crate::components::{AppGlobalsResource, BasePageContainer, Initialized, LoadingSpinner};
use crate::routing;

#[must_use]
#[component(transparent)]
pub fn LeaAuthenticated(
    children: ChildrenFn,
    #[prop(optional, into)] loading: ViewFnOnce,
    #[prop(optional, into)] unauthenticated: ViewFn,
    #[prop(optional, into)] disabled_auth: ViewFn,
) -> impl IntoView {
    let auth = use_app_globals().auth;
    let children = StoredValue::new(children);

    match auth {
        Authentication::Enabled(auth) => {
            let unauthenticated = move || unauthenticated.run();
            let authenticated = move || auth.get().is_authenticated();

            Either::Right(view! {
                <Transition fallback=loading>
                    <Show
                        when=authenticated
                        fallback=unauthenticated
                    >
                        { children.read_value()() }
                    </Show>
                </Transition>
            }.into_any())
        }
        Authentication::Disabled => {
            tracing::warn!("Warning: Authentication disabled - Neither an authentication config provided, nor is the user authenticated.");
            Either::Left(disabled_auth.run())
        }
    }
}


#[component]
pub fn LoginPage(app_globals: AppGlobalsResource) -> impl IntoView {

    view! {
        <BasePageContainer
            title="Login page"
            breadcrumbs=Vec::new()
            controls=|| ()
        >
            <Initialized
                app_globals
                authentication_required=false
            >
                <LeaAuthenticated
                    unauthenticated=move || {
                        view! {
                            <p class="subtitle">"Please sign in."</p>
                            <LoginLink class="button">
                                <span class="ml-2 is-size-6">"Sign in"</span>
                            </LoginLink>
                            }
                    }
                    disabled_auth=move || {
                        view! {
                            <p class="subtitle">"Authentication disabled."</p>
                            <a href=routing::path::dashboard class="button">
                                <span class="ml-2 is-size-6">"Go to Dashboard"</span>
                            </a>
                        }
                    }
                    loading=LoadingSpinner
                >
                    <p class="subtitle">"Authenticated"</p>
                    <LogoutLink class="button">
                        <span class="ml-1 is-size-6">"Sign out"</span>
                    </LogoutLink>
                </LeaAuthenticated>
            </Initialized>
            
        </BasePageContainer>
    }
}
