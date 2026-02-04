use leptos::prelude::*;
use opendut_lea_components::health::{self, Health};
use opendut_model::peer::state::{PeerConnectionState, PeerState};

#[component]
pub fn PeerHealth(
    #[prop(into)] state: Signal<PeerState>
) -> impl IntoView {

    let health_state = Signal::derive(move || {
        state.with(|peer_state| {
            match peer_state.connection {
                PeerConnectionState::Offline => health::State {
                    kind: health::StateKind::Unknown,
                    text: String::from("Disconnected"),
                },
                PeerConnectionState::Online { .. } => health::State {
                    kind: health::StateKind::Green,
                    text: String::from("Connected. No errors."),
                },
            }
        })
    });

    view! {
        <Health state=health_state />
    }
}
