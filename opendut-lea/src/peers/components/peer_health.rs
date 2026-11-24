use crate::util::client_error_description;
use std::sync::Arc;
use leptos::prelude::*;
use tracing::error;
use opendut_lea_components::{ErrorText, FontAwesomeIcon, LoadingSpinner};
use opendut_lea_components::health::{self, Health};
use opendut_lea_components::tooltip::Tooltip;
use opendut_model::peer::PeerId;
use opendut_model::peer::state::{PeerConnectionState, PeerState};
use crate::app::use_app_globals;

#[component]
pub fn PeerHealth(
    peer_id: Signal<PeerId>,
) -> impl IntoView {

    let globals = use_app_globals();

    let state: LocalResource<Result<PeerState, ErrorText>> = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        async move {
            let peer_id = peer_id.get();
            carl.peers.get_peer_state(peer_id).await
                .map_err(|error| {
                    error!("Failed to request the peer state for peer <{peer_id}>: {error}");

                    let error_description = client_error_description(
                        &error,
                        "The peer state could not be loaded. Please check your connection and try again.",
                    );

                    ErrorText {
                        headline: String::from("Unable to load peer state"),
                        description: error_description,
                    }
                })
        }
    });
    
    move || match state.get() {
        Some(Ok(state)) => {
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
        Some(Err(_)) => {
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
        None => {
            view! {
                <div class="is-flex is-align-items-center is-justify-content-center">
                    <LoadingSpinner />
                </div>
            }.into_any()
        }
    }
}
