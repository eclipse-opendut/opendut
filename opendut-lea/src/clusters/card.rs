use leptos::*;
use crate::app::{ExpectGlobals, use_app_globals};

use crate::clusters::components::CreateClusterButton;
use crate::routing;

#[derive(Clone)]
struct Clusters {
    deployed: usize,
    undeployed: usize,
}

#[component]
pub fn ClustersCard() -> impl IntoView {

    let globals = use_app_globals();

    let clusters: Resource<(), Clusters> = create_local_resource(|| {}, move |_| {
        let mut carl = globals.expect_client();
        async move {
            let configured = carl.cluster.list_cluster_configurations().await
                .expect("Failed to request the list of cluster configurations.")
                .len();
            let deployed = carl.cluster.list_cluster_deployments().await
                .expect("Failed to request the list of cluster deployments.")
                .len();
            Clusters {
                deployed,
                undeployed: configured.saturating_sub(deployed)
            }
        }
    });

    view! {
        <div class="card">
            <div class="card-header">
                <a class="card-header-title has-text-link" href=routing::path::clusters_overview>"Clusters"</a>
            </div>
            <div class="card-content">
                <div class="level">
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">Deployed</p>
                            <p class="title">
                                <Suspense
                                    fallback={ move || view! { <span>"-"</span> }}
                                >
                                    <span>{ move || clusters.get().map(|peers| peers.deployed) }</span>
                                </Suspense>
                            </p>
                        </div>
                    </div>
                    <div class="level-item has-text-centered">
                        <div>
                            <p class="heading">Undeployed</p>
                            <p class="title">
                                <Suspense
                                    fallback={ move || view! { <span>"-"</span> }}
                                >
                                    <span>{ move || clusters.get().map(|peers| peers.undeployed) }</span>
                                </Suspense>
                            </p>
                        </div>
                    </div>
                </div>
            </div>
            <div class="card-footer">
                <div class="m-2">
                    <CreateClusterButton />
                </div>
            </div>
        </div>
    }
}
