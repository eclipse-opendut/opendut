use leptos::{ChildrenFn, component, IntoView, Show, Transition, view, ViewFn};

use crate::app::{ExpectGlobals, use_app_globals};

#[must_use]
#[component(transparent)]
pub fn LeaAuthenticated(
    children: ChildrenFn,
    #[prop(optional, into)] loading: ViewFn,
    #[prop(optional, into)] unauthenticated: ViewFn,
    #[prop(optional, into)] disabled_auth: ViewFn,
) -> impl IntoView {
    let auth = use_app_globals().expect_auth();
    match auth {
        None => {
            disabled_auth.run()
        }
        Some(auth) => {
            let unauthenticated = move || unauthenticated.run();
            let authenticated = move || auth.authenticated();

            view! {
                <Transition fallback=loading>
                    <Show
                        when=authenticated.clone()
                        fallback=unauthenticated.clone()
                        children=children.clone()
                    />
                </Transition>
            }
        }
    }
}
