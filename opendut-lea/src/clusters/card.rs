use leptos::prelude::*;
use opendut_lea_components::ErrorText;
use crate::dashboard::Clusters;
use crate::dashboard::components::{DashboardCard, DashboardCardItem};
use crate::clusters::components::CreateClusterButton;
use crate::routing;

#[component]
pub fn ClustersCard(clusters: Signal<Option<Result<Clusters, ErrorText>>>) -> impl IntoView {

    let deployed = Signal::derive(move || {
        clusters.get().map(|result| result.map(|clusters| clusters.deployed))
    });
    let undeployed = Signal::derive(move || {
        clusters.get().map(|result| result.map(|clusters| clusters.undeployed))
    });

    view! {
        <DashboardCard
            title="Clusters"
            href=routing::path::clusters_overview
            footer=view! { <CreateClusterButton /> }
        >
            <DashboardCardItem title="Deployed" value=deployed />
            <DashboardCardItem title="Undeployed" value=undeployed />
        </DashboardCard>
    }
}
