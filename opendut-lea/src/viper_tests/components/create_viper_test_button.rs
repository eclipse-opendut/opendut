use leptos::prelude::*;

use leptos_router::hooks::use_navigate;
use opendut_model::viper::ViperTestId;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn CreateViperTestButton() -> impl IntoView {

    let use_navigate = use_navigate();

    view! {
        <IconButton
            icon=FontAwesomeIcon::Plus
            color=ButtonColor::Success
            size=ButtonSize::Normal
            state=ButtonState::Enabled
            label="Create VIPER Test"
            on_action=move || {
                navigate_to(WellKnownRoutes::ViperTestConfigurator {
                    id: ViperTestId::random()
                }, use_navigate.clone());
            }
        />
    }
}
