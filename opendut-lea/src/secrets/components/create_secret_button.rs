use leptos::prelude::*;

use leptos_router::hooks::use_navigate;
use opendut_model::secret::SecretId;
use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn CreateSecretButton() -> impl IntoView {

    let use_navigate = use_navigate();

    view! {
        <IconButton
            icon=FontAwesomeIcon::Plus
            color=ButtonColor::Success
            size=ButtonSize::Normal
            state=ButtonState::Enabled
            label="Create secret"
            on_action=move || {
                navigate_to(WellKnownRoutes::SecretConfigurator {
                    id: SecretId::random()
                }, use_navigate.clone());
            }
        />
    }
}
