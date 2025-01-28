use leptos::either::Either;
use leptos::prelude::*;
use opendut_auth::public::Authentication;

use crate::app::use_app_globals;


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

