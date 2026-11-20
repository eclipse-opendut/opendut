use std::sync::Arc;
use leptos::prelude::*;
use opendut_lea_components::{ErrorText, FontAwesomeIcon};
use opendut_lea_components::tooltip::{Tooltip, TooltipContent};
use crate::dashboard::Peers;
use crate::peers::components::CreatePeerButton;
use crate::routing;

#[component]
pub fn PeersCard(peers: LocalResource<Result<Peers, ErrorText>>) -> impl IntoView {

    let online = Signal::derive(move || {
        peers.get().map(|result| result.map(|peers| peers.online))
    });
    let offline = Signal::derive(move || {
        peers.get().map(|result| result.map(|peers| peers.offline))
    });

    view! {
        <div class="card">
            <div class="card-header">
                <a class="card-header-title has-text-link" href=routing::path::peers_overview>"Peers"</a>
            </div>
            <div class="card-content">
                <div class="level">
                    <PeerCardItem title="Online" value=online />
                    <PeerCardItem title="Offline" value=offline />
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

#[component]
fn PeerCardItem(
    #[prop(into)] title: String,
    value: Signal<Option<Result<usize, ErrorText>>>,
) -> impl IntoView {

    view! {
        <div class="level-item has-text-centered">
            <div>
                <p class="heading">{title.clone()}</p>
                <div class="title">
                    <Suspense fallback=move || view! { <span>"-"</span> }>
                        {move || match value.get() {
                            None => view! { <span>"-"</span> }.into_any(),
                            Some(Ok(value)) => view! { <span>{value}</span> }.into_any(),
                            Some(Err(error)) => {
                                let error_label = format!("Could not load {}", title);
                                let tooltip: TooltipContent = Arc::new(move || {
                                    view! {
                                        <span class="has-text-left has-text-weight-normal">
                                            <p> { error.headline.clone() } </p>
                                            <p> { error.description.clone() } </p>
                                        </span>
                                    }.into_any()
                                });

                                view! {
                                    <Tooltip text=tooltip>
                                        <span
                                            class="has-text-danger"
                                            role="img"
                                            aria-label=error_label
                                        >
                                            <i class=FontAwesomeIcon::TriangleExclamation.as_class() aria-hidden="true" />
                                        </span>
                                    </Tooltip>
                                }.into_any()
                            }
                        }}
                    </Suspense>
                </div>
            </div>
        </div>
    }
}
