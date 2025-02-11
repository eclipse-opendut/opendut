use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerDescriptor;

use crate::app::use_app_globals;
use crate::clusters::configurator::components::{DeviceSelection, DeviceSelector, LeaderSelection};
use crate::clusters::configurator::components::Controls;
use crate::clusters::configurator::tabs::{DevicesTab, GeneralTab, LeaderTab, TabIdentifier};
use crate::clusters::configurator::types::UserClusterConfiguration;
use crate::clusters::overview::IsDeployed;
use crate::components::{use_active_tab, BasePageContainer, Breadcrumb, LoadingSpinner};
use crate::components::UserInputValue;

mod types;
mod tabs;
mod components;

#[component(transparent)]
pub fn ClusterConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let cluster_id = Signal::derive(move || params.with(|params| {
        params.get("id")
            .and_then(|id| ClusterId::try_from(id.as_str()).ok())
    }).unwrap_or_else(ClusterId::random));

    let cluster_configuration = RwSignal::new(
        UserClusterConfiguration {
            id: cluster_id.get_untracked(),
            name: UserInputValue::Left(String::from("Enter a valid cluster name.")),
            devices: DeviceSelection::Left(String::from("Select at least two devices.")),
            leader: LeaderSelection::Left(String::from("Select a leader.")),
        }
    );

    {
        let carl = globals.client.clone();

        LocalResource::new(move || {
            let cluster_id = cluster_id.get();
            let mut carl = carl.clone();

            async move {
                if let Ok(configuration) = carl.cluster.get_cluster_configuration(cluster_id).await {
                    cluster_configuration.set(
                        UserClusterConfiguration {
                            id: cluster_id,
                            name: UserInputValue::Right(configuration.name.value()),
                            devices: DeviceSelection::Right(configuration.devices),
                            leader: LeaderSelection::Right(configuration.leader),
                        }
                    )
                } else {
                    tracing::debug!("Could not find cluster with ID <{cluster_id}> in backend. Showing empty configurator to create new cluster.");
                }
            }
        });
    }

    let peer_descriptors = get_all_peers();

    view! {
        <Transition
            fallback=LoadingSpinner
        >
        {move || Suspend::new(async move {
            let peers = RwSignal::new(peer_descriptors.await).read_only();

            view! {
                <LoadedClusterConfigurator cluster_configuration peers />
            }
        })}
        </Transition>
    }
}

#[component]
fn LoadedClusterConfigurator(
    cluster_configuration: RwSignal<UserClusterConfiguration>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {
    let globals = use_app_globals();

    let cluster_id = Signal::derive(move || cluster_configuration.get().id);

    let breadcrumbs = Signal::derive(move || {
        let cluster_id = cluster_id.get().0.to_string();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Clusters", "clusters"),
            Breadcrumb::new(Clone::clone(&cluster_id), cluster_id),
        ]
    });

    let active_tab = use_active_tab::<TabIdentifier>();

    let cluster_deployments = {
        let carl = globals.client.clone();
        LocalResource::new(move || {
            let mut carl = carl.clone();
            async move {
                carl.cluster.list_cluster_deployments().await
                    .expect("Failed to request the list of cluster deployments")
            }
        })
    };


    let deployed_clusters = Signal::derive(move || {
        match cluster_deployments.get() {
            Some(deployed_clusters) => {
                deployed_clusters.iter().map(|cluster_deployment| cluster_deployment.id).collect::<Vec<_>>()
            }
            None => Vec::new()
        }
    });

    let deployed_signal = Signal::derive(move || IsDeployed(
        deployed_clusters.get().contains(&cluster_id.get())
    ));

    view! {
        <BasePageContainer
            title="Configure Cluster"
            breadcrumbs
            controls=move || {
                view! {
                    <Controls
                        cluster_configuration=cluster_configuration.read_only()
                        deployed_signal
                    />
                }
            }
        >
                <div class="tabs">
                    <ul>
                        <li class=("is-active", move || TabIdentifier::General == active_tab.get())>
                            <a href={ TabIdentifier::General.as_str() }>General</a>
                        </li>
                        <li class=("is-active", move || TabIdentifier::Devices == active_tab.get())>
                            <a href={ TabIdentifier::Devices.as_str() }>Devices</a>
                        </li>
                        <li class=("is-active", move || TabIdentifier::Leader == active_tab.get())>
                            <a href={ TabIdentifier::Leader.as_str() }>Leader</a>
                        </li>
                    </ul>
                </div>

            <fieldset prop:disabled=move || { deployed_signal.get().0 }>
                <div class="container">
                    <div class=("is-hidden", move || TabIdentifier::General != active_tab.get())>
                        <GeneralTab cluster_configuration />
                    </div>
                    <div class=("is-hidden", move || TabIdentifier::Devices != active_tab.get())>
                        <DevicesTab cluster_configuration peers />
                    </div>
                    <div class=("is-hidden", move || TabIdentifier::Leader != active_tab.get())>
                        <LeaderTab cluster_configuration peers />
                    </div>
                </div>
            </fieldset>
        </BasePageContainer>
    }
}


pub(super) fn get_all_peers() -> LocalResource<Vec<PeerDescriptor>> {
    let globals = use_app_globals();

    LocalResource::new(move || {
        let mut carl = globals.client.clone();
        async move {
            carl.peers.list_peer_descriptors().await
                .expect("Failed to request the list of all peers.")
        }
    })
}
