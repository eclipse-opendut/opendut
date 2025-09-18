use leptos::prelude::*;
use leptos_oidc::{LoginLink, LogoutLink};
use crate::components::{AppGlobalsResource, Initialized, LeaAuthenticated};
use crate::routing;
use crate::user::UserAuthenticationSignal;

#[component]
pub fn ProfileSidebar(app_globals: AppGlobalsResource, profile_visible: RwSignal<bool>) -> impl IntoView {
    
    view! {
        <Initialized app_globals authentication_required=false >
            <aside class="dut-menu is-right column" class:is-active= move || profile_visible.get() >
                <ul class="dut-menu-list">
                    <LeaAuthenticated
                        unauthenticated=move || {
                            view! {
                                <LoginLink>
                                    <span class="is-size-6">"Sign in"</span>
                                </LoginLink>

                            }
                        }
                        disabled_auth=move || {
                            view! {
                                <a href=routing::path::dashboard>
                                    <span class="is-size-6">"Sign in"</span>
                                </a>
                            }
                        }>
                        <LoggedInUser />
                        <LogoutLink>
                            <span class="ml-1 is-size-6">"Sign out"</span>
                        </LogoutLink>
                    </LeaAuthenticated>
                </ul>
            </aside>
        </Initialized>
    }
}

#[component]
pub fn LoggedInUser() -> impl IntoView {

    let user = use_context::<UserAuthenticationSignal>().expect("UserAuthenticationSignal should be provided in the context.");
    let user_name  = move || { user.get().username() };

    view! {
        <span class="ml-1 is-size-6">"Logged in as: " { user_name }</span>
        <a href=routing::path::user class="dut-nav-flyout-item">
            <span class="ml-1 is-size-6">"Profile"</span>
        </a>
    }
}