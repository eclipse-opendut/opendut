use leptos::prelude::*;
use opendut_lea_components::ErrorText;
use crate::dashboard::Peers;
use crate::dashboard::components::{DashboardCard, DashboardCardItem};
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
        <DashboardCard
            title="Peers"
            href=routing::path::peers_overview
            footer=view! { <CreatePeerButton /> }
        >
            <DashboardCardItem title="Online" value=online />
            <DashboardCardItem title="Offline" value=offline />
        </DashboardCard>
    }
}
