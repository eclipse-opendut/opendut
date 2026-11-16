use std::sync::Arc;
use leptos::prelude::*;
use opendut_lea_components::FontAwesomeIcon;
use opendut_lea_components::health::{self, Health};
use opendut_lea_components::tooltip::Tooltip;
use opendut_model::peer::state::{PeerConnectionState, PeerState};

#[component]
pub fn PeerHealth(
    #[prop(into)] state: Signal<Option<PeerState>>
) -> impl IntoView {
    
    match state.get() {
        Some(state) => {
            let health_state = Signal::derive(move || {
                match state.connection {
                    PeerConnectionState::Offline => health::State {
                        kind: health::StateKind::Unknown,
                        text: String::from("Disconnected"),
                    },
                    PeerConnectionState::Online { .. } => health::State {
                        kind: health::StateKind::Green,
                        text: String::from("Connected. No errors."),
                    },
                }
            });

            view! {
                <Health state=health_state />
            }.into_any()
        }
        None => {
            let tooltip_content = Arc::new(move || {
                view! {
                    Unable to load peer states.
                }.into_any()
            });

            view! {
                <div class="is-flex is-align-items-center is-justify-content-center">
                    <Tooltip text=tooltip_content>
                        <i class=format!("has-text-danger fa-lg {}", FontAwesomeIcon::TriangleExclamation.as_class()) />
                    </Tooltip>
                </div>
            }.into_any()
        }
    }
}
