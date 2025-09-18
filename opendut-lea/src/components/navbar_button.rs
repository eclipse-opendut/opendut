use leptos::prelude::*;

use leptos_router::hooks::{use_location, use_navigate};
use opendut_lea_components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn NavbarButton(
    #[prop(into)] icon: Signal<FontAwesomeIcon>,
    #[prop(into)] label: Signal<String>,
    #[prop(into)] route: Signal<WellKnownRoutes>,
    #[prop(into)] path: Signal<String>,
) -> impl IntoView {
    let navigate = use_navigate();
    let location = use_location();
    let is_active = move || location.pathname.get() == path.get();

    view! {
        <div
            class="navbar-item px-0 mx-1"
            class:is-active= is_active
            class:is-tab= is_active
        >
            <IconButton
                icon
                color=ButtonColor::Light
                size=ButtonSize::Normal
                state=ButtonState::Enabled
                label
                show_label=true
                on_action=move || {
                    navigate_to(
                        route.get(),
                        Clone::clone(&navigate)
                    );
                }
            />


                // <span class="icon-text has-text-dark">
                //     <span class="icon">
                //         <i class=icon.get().as_class()></i>
                //     </span>
                //     <span>{label}</span>
                // </span>
        </div>
    }
}