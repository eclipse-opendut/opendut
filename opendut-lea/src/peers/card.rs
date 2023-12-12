use leptos::*;

use crate::api::use_carl;

#[derive(Clone)]
struct Peers {
    online: usize,
    offline: usize,
}

#[component]
pub fn PeersCard() -> impl IntoView {

    let carl = use_carl();

    let peers: Resource<(), Peers> = create_local_resource(|| {}, move |_| {
        async move {
            let mut carl = carl.get_untracked();
            let registered = carl.peers.list_peers().await
                .expect("Failed to request the list of peers.")
                .len();
            let connected = carl.broker.list_peers().await
                .expect("Failed to request the list of connected peers.")
                .len();
            Peers {
                offline: registered.saturating_sub(connected), // TODO: A simple `sub` may fail, due to a bug in the registration/un-registration process of CARL, there can be connected peers that are not registered.
                online: connected
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
        </div>
    }
}
