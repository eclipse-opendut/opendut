use leptos::{component, create_node_ref, create_rw_signal, IntoView, MaybeSignal, SignalGet, SignalSet, SignalUpdate, view};
use leptos::html::Div;
use leptos_oidc::components::{LoginLink, LogoutLink};
use leptos_use::on_click_outside;

use crate::components::{LeaAuthenticated, ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton, Initialized};
use crate::use_context;
use opendut_carl_api::carl::wasm::OptionalAuthData;
use crate::{WriteSignal, ReadSignal};

#[component(transparent)]
pub fn Navbar() -> impl IntoView {
    #[component]
    fn inner() -> impl IntoView {
        let menu_visible = create_rw_signal(false);
        let profile_visible = create_rw_signal(false);

        let menu_button_icon = MaybeSignal::derive(move || {
            if menu_visible.get() {
                FontAwesomeIcon::XMark
            } else {
                FontAwesomeIcon::Bars
            }
        });

        let profile_button_icon = MaybeSignal::derive(move || {
            if profile_visible.get() {
                FontAwesomeIcon::XMark
            } else {
                FontAwesomeIcon::User
            }
        });

        let menu_button_area = create_node_ref::<Div>();
        let _ = on_click_outside(menu_button_area, move |_| {
            menu_visible.set(false)
        });

        let profile_button_area = create_node_ref::<Div>();
        let _ = on_click_outside(profile_button_area, move |_| {
            profile_visible.set(false)
        });

        view! {
            <div class="columns is-vcentered px-3 pt-3 mb-4 has-background-light is-mobile">
                <div class="column is-narrow">
                    <div class="dut-nav-flyout" class=("is-active", move || menu_visible.get())>
                        <div node_ref=menu_button_area class="dropdown-trigger">
                            <IconButton
                                icon=menu_button_icon
                                color=ButtonColor::Light
                                size=ButtonSize::Normal
                                state=ButtonState::Enabled
                                label="User"
                                on_action=move || menu_visible.update(|is_visible| *is_visible = !*is_visible)
                            />
                        </div>
                        <div class="dut-nav-flyout-container mt-2 has-background-light left--3">
                            <div class="dut-nav-flyout-content">
                                <div>
                                    <a class="dut-nav-flyout-item" href="/">
                                        <i class="fa-solid fa-gauge-high fa-lg pr-1" />
                                        <span class="ml-2 is-size-6">"Dashboard"</span>
                                    </a>
                                    <a class="dut-nav-flyout-item" href="/clusters">
                                        <i class="fa-solid fa-circle-nodes fa-lg pr-1" />
                                        <span class="ml-2 is-size-6">"Clusters"</span>
                                    </a>
                                    <a class="dut-nav-flyout-item" href="/peers">
                                        <i class="fa-solid fa-microchip fa-lg pr-1" />
                                        <span class="ml-2 is-size-6">"Peers"</span>
                                    </a>
                                </div>
                                <div>
                                    <hr class="dut-nav-flyout-divider" />
                                    <div class="px-2">
                                        <a class="is-size-7" href="/about">"About"</a>
                                    </div>
                                    <div class="px-2">
                                        <a class="is-size-7" href="/licenses">"Licenses"</a>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
                <div class="column">
                    <a class="" href="/"><span class="dut-title is-3">"openDuT"</span></a>
                </div>
                <div class="column is-narrow">
                    <div class="dut-nav-flyout is-right" class=("is-active", move || profile_visible.get())>
                        <div node_ref=profile_button_area class="dropdown-trigger">
                            <IconButton
                                icon=profile_button_icon
                                color=ButtonColor::Light
                                size=ButtonSize::Normal
                                state=ButtonState::Enabled
                                label="User"
                                on_action=move || profile_visible.update(|is_visible| *is_visible = !*is_visible)
                            />
                        </div>
                        <div class="dut-nav-flyout-container mt-2 has-background-light right--3">
                            <div class="dut-nav-flyout-content">
                                <div>
                                    <LeaAuthenticated unauthenticated=move || {
                                        view! {
                                            <LoginLink class="dut-nav-flyout-item">
                                                <span class="ml-2 is-size-6">"Sign in"</span>
                                            </LoginLink>

                                        }
                                        }
                                        disabled_auth=move || {
                                            view! {
                                                <a href="/" class="dut-nav-flyout-item">
                                                    <span class="ml-2 is-size-6">"Sign in"</span>
                                                </a>
                                            }
                                        }>
                                        <LoggedInUser></LoggedInUser>
                                        <LogoutLink class="dut-nav-flyout-item">
                                            <span class="ml-1 is-size-6">"Sign out"</span>
                                        </LogoutLink>
                                    </LeaAuthenticated>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        }
    }

    view! {
        <Initialized
            authentication_required=false
        >
            <Inner />
        </Initialized>
    }
}

#[component]
pub fn LoggedInUser() -> impl IntoView {

    let (auth_data, _) = use_context::<(ReadSignal<OptionalAuthData>, WriteSignal<OptionalAuthData>)>().expect("AuthData should be provided in the context.");
    let user_name  = move || {
        match auth_data.get().auth_data {
            None => { "Unknown User".to_string() }
            Some(auth_data) => { auth_data.preferred_username }
        }
    };

    view! {
        <span class="ml-1 is-size-6">"Logged in as: " { user_name }</span>
        <a href="/user" class="dut-nav-flyout-item">
            <span class="ml-1 is-size-6">"Profile"</span>
        </a>
    }
}