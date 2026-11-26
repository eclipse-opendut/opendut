use crate::util::client_error_description;
use leptos::prelude::*;
use tracing::error;
use opendut_lea_components::{ErrorIndicator, ErrorText, LoadingSpinner};
use opendut_lea_components::health::{self, Health};
use opendut_model::peer::PeerId;
use opendut_model::peer::state::PeerConnectionState;
use crate::api::peers::get_peer_state;

#[component]
pub fn PeerHealth(
    peer_id: Signal<PeerId>,
) -> impl IntoView {

    let state = get_peer_state(peer_id, None);

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
        Some(Err(error)) => {
            error!("Failed to request the peer state: {error}");
            let error = ErrorText {
                headline: String::from("Unable to load peer state"),
                description: client_error_description(
                    error.as_ref(),
                    "The peer state could not be loaded. Please check your connection and try again.",
                ),
            };
            view! {
                <div class="is-flex is-align-items-center is-justify-content-center">
                    <ErrorIndicator text=error />
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
