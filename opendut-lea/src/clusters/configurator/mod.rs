use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use opendut_lea_components::tabs::{Tab, Tabs};
use crate::components::UserInputValue;
use opendut_model::cluster::ClusterId;
use opendut_model::peer::PeerDescriptor;

use crate::app::use_app_globals;
use crate::clusters::configurator::components::{DeviceSelection, DeviceSelector, LeaderSelection};
use crate::clusters::configurator::components::Controls;
use crate::clusters::configurator::tabs::{DevicesTab, GeneralTab, LeaderTab, TabIdentifier};
use types::UserClusterDescriptor;
use crate::clusters::IsDeployed;
use crate::components::{use_active_tab, BasePageContainer, Breadcrumb, LoadingSpinner};

mod tabs;
mod components;
pub mod types;

#[component(transparent)]
pub fn ClusterConfigurator() -> impl IntoView {
    let globals = use_app_globals();
    let params = use_params_map();

    let cluster_id = Memo::new(move |_| params.with(|params| {
        params.get("id")
            .and_then(|id| ClusterId::try_from(id.as_str()).ok())
    }).unwrap_or_else(ClusterId::random));

    let cluster_descriptor = RwSignal::new(
        UserClusterDescriptor {
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
                if let Ok(configuration) = carl.cluster.get_cluster_descriptor(cluster_id).await {
                    cluster_descriptor.set(
                        UserClusterDescriptor {
                            id: cluster_id,
                            name: UserInputValue::Right(configuration.name.value().to_owned()),
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
                <LoadedClusterConfigurator cluster_descriptor peers />
            }
        })}
        </Transition>
    }
}

#[component]
fn LoadedClusterConfigurator(
    cluster_descriptor: RwSignal<UserClusterDescriptor>,
    peers: ReadSignal<Vec<PeerDescriptor>>,
) -> impl IntoView {
    let globals = use_app_globals();

    let cluster_id = Signal::derive(move || cluster_descriptor.get().id);

    let breadcrumbs = Signal::derive(move || {
        let cluster_id = cluster_id.get().uuid.to_string();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Clusters", "clusters"),
            Breadcrumb::new(Clone::clone(&cluster_id), cluster_id),
        ]
    });

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

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = cluster_descriptor.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {
        vec![
            Tab {
                title: String::from("General"),
                href: TabIdentifier::General.as_str().to_owned(),
                is_error: Signal::derive(move || !cluster_descriptor.read().name.is_right())
            },
            Tab {
                title: String::from("Devices"),
                href: TabIdentifier::Devices.as_str().to_owned(),
                is_error: Signal::derive(move || !cluster_descriptor.read().devices.is_right())
            },
            Tab {
                title: String::from("Leader"),
                href: TabIdentifier::Leader.as_str().to_owned(),
                is_error: Signal::derive(move || !cluster_descriptor.read().leader.is_right())
            }
        ]
    });

    let active_tab = use_active_tab::<TabIdentifier>();

    view! {
        <BasePageContainer
            title="Configure Cluster"
            subtitle
            breadcrumbs
            controls=move || {
                view! {
                    <Controls
                        cluster_descriptor=cluster_descriptor.read_only()
                        deployed_signal
                    />
                }
            }
        >
            <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                <fieldset prop:disabled=move || { deployed_signal.get().0 }>
                    { move || match active_tab.get() {
                        TabIdentifier::General => view! { <GeneralTab cluster_descriptor /> }.into_any(),
                        TabIdentifier::Devices => view! { <DevicesTab cluster_descriptor peers /> }.into_any(),
                        TabIdentifier::Leader => view! { <LeaderTab cluster_descriptor peers /> }.into_any()
                    }}
                </fieldset>
            </Tabs>
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
