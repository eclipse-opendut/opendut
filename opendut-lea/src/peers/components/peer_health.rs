use leptos::prelude::*;
use opendut_lea_components::{ErrorIndicator, LoadingSpinner};
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
