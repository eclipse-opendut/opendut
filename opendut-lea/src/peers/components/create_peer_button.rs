use leptos::prelude::*;

use opendut_types::peer::PeerId;

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn CreatePeerButton() -> impl IntoView {

    view! {
        <IconButton
            icon=FontAwesomeIcon::Plus
            color=ButtonColor::Success
            size=ButtonSize::Normal
            state=ButtonState::Enabled
            label="Create peer"
            on_action=move || {
                navigate_to(WellKnownRoutes::PeerConfigurator {
                    id: PeerId::random()
                });
            }
        />
    }
}
