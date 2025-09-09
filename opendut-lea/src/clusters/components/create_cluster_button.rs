use leptos::prelude::*;

use leptos_router::hooks::use_navigate;
use opendut_model::cluster::ClusterId;

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn CreateClusterButton() -> impl IntoView {

    let navigate = use_navigate();

    view! {
        <IconButton
            icon=FontAwesomeIcon::Plus
            color=ButtonColor::Success
            size=ButtonSize::Normal
            state=ButtonState::Enabled
            label="Create Cluster"
            on_action=move || {
                navigate_to(
                    WellKnownRoutes::ClusterConfigurator {
                        id: ClusterId::random()
                    },
                    navigate.clone()
                );
            }
        />
    }
}
