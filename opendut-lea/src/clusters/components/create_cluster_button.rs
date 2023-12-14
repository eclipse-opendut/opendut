use leptos::*;

use opendut_types::cluster::ClusterId;

use crate::components::{ButtonColor, ButtonSize, ButtonState, FontAwesomeIcon, IconButton};
use crate::routing::{navigate_to, WellKnownRoutes};

#[component]
pub fn CreateClusterButton() -> impl IntoView {

    view! {
        <IconButton
            icon=FontAwesomeIcon::Plus
            color=ButtonColor::Success
            size=ButtonSize::Normal
            state=ButtonState::Enabled
            label="Create Cluster"
            on_action=move || {
                navigate_to(WellKnownRoutes::ClusterConfigurator {
                    id: ClusterId::random()
                });
            }
        />
    }
}
