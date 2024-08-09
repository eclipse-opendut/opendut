use leptos::*;
use opendut_types::peer::state::PeerState;
use crate::app::{ExpectGlobals, use_app_globals};
use crate::peers::components::CreatePeerButton;

#[derive(Clone)]
struct Peers {
    online: usize,
    offline: usize,
}

#[component]
pub fn PeersCard() -> impl IntoView {

    let globals = use_app_globals();

    let peers: Resource<(), Peers> = create_local_resource(|| {}, move |_| {
        let mut carl = globals.expect_client();
        async move {
            let registered = carl.peers.list_peer_descriptors().await
                .expect("Failed to request the list of peers.");

            let mut online_counter = 0;
            let mut offline_counter = 0;
            
            for peer in registered {
                let peer_state = carl.peers.get_peer_state(peer.id).await.expect("Failed to request state of peer.");
                match peer_state {
                    PeerState::Down => { offline_counter += 1 }
                    PeerState::Up { .. } => { online_counter += 1}
                }
            };
            
            Peers {
                offline: offline_counter,
                online: online_counter
            }
        }
    });

    view! {
        <div class="card">
            <div class="card-header">
                <a class="card-header-title has-text-link" href="/peers">"Peers"</a>
            </div>
            <div class="card-content">
                <div class="level">
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">Online</p>
                            <p class="title">
                                <Suspense
                                    fallback={ move || view! { <span>"-"</span> }}
                                >
                                    <span>{ move || peers.get().map(|peers| peers.online) }</span>
                                </Suspense>
                            </p>
                        </div>
                    </div>
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">Offline</p>
                            <p class="title">
                                <Suspense
                                    fallback={ move || view! { <span>"-"</span> }}
                                >
                                    <span>{ move || peers.get().map(|peers| peers.offline) }</span>
                                </Suspense>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
            <div class="card-footer">
                <div class="m-2">
                    <CreatePeerButton />
                </div>
            </div>
        </div>
    }
}
