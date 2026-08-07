use leptos::prelude::*;
use opendut_lea_components::health::{self, Health};
use opendut_lea_components::tooltip::TooltipDirection;
use opendut_model::cluster::state::ClusterState;

#[component]
pub fn ClusterHealth(
    #[prop(into)] state: Signal<ClusterState>,
    #[prop(optional)] tooltip_direction: TooltipDirection
) -> impl IntoView {
    let _ = state;

    let health_state = Signal::derive(move || {
        health::State {
            //TODO implement Cluster health in backend and display it here
            kind: health::StateKind::Unknown,
            text: String::from("Unknown"),
        }
    });

    view! {
        <Health
            state=health_state
            tooltip_direction
        />
    }
}
