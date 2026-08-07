use crate::app::use_app_globals;
use crate::components::use_active_tab;
use crate::components::{BasePageContainer, Breadcrumb, UserInputError, UserInputValue};
use crate::peers::configurator::components::Controls;
use crate::peers::configurator::tabs::{DevicesTab, ExecutorTab, GeneralTab, NetworkTab, SetupTab, TabIdentifier};
use crate::routing::{navigate_to, WellKnownRoutes};
use crate::util;
use leptos::either::Either;
use leptos::prelude::*;
use opendut_model::peer::executor::{ExecutorDescriptor, ExecutorKind};
use opendut_model::peer::PeerId;
use leptos_router::hooks::{use_navigate, use_params_map};
use opendut_lea_components::LoadingSpinner;
use opendut_lea_components::tabs::{Tab, Tabs};
use crate::peers::configurator::types::devices::UserDeviceConfiguration;
use crate::peers::configurator::types::executor::{UserContainerEnv, UserPeerExecutor, UserPeerExecutorKind};
use crate::peers::configurator::types::network::{UserNetworkInterface, UserPeerNetwork};
use crate::peers::configurator::types::UserPeerDescriptor;
use opendut_model::peer::state::PeerState;

mod components;
mod tabs;
mod types;

#[component(transparent)]
pub fn PeerConfigurator() -> impl IntoView {

    let globals = use_app_globals();
    let params = use_params_map();

    let active_tab = use_active_tab::<TabIdentifier>();

    let peer_id = {
        let peer_id = params.with_untracked(|params| {
            params.get("id").and_then(|id| PeerId::try_from(id.as_str()).ok())
        });
        match peer_id {
            None => {
                let use_navigate = use_navigate();
                navigate_to(WellKnownRoutes::ErrorPage {
                    title: String::from("Invalid PeerId"),
                    text: String::from("Could not parse the provided value as PeerId!"),
                    details: None,
                }, use_navigate);
                PeerId::random()
            }
            Some(peer_id) => {
                peer_id
            }
        }
    };

    let peer_state = RwSignal::new(PeerState::default());

    let user_peer_descriptor = RwSignal::new(UserPeerDescriptor {
        id: peer_id,
        name: UserInputValue::Left(UserInputError::from("Enter a valid peer name.")),
        location: UserInputValue::Right(String::from("")),
        devices: Vec::new(),
        network: UserPeerNetwork {
            network_interfaces: Vec::new(),
            bridge_name: UserInputValue::Right(String::from("")),
        },
        is_new: true,
        executors: Vec::new(),
    });

    let peer_descriptor_resource = LocalResource::new(move || {
        let mut carl = globals.client.clone();
        async move {
            if let Ok(configuration) = carl.peers.get_peer_descriptor(peer_id).await {
                let clusters = carl.cluster.list_cluster_descriptors().await
                    .unwrap_or(vec![]);

                user_peer_descriptor.update(|user_configuration| {
                    user_configuration.name = UserInputValue::Right(configuration.name.value().to_owned());
                    user_configuration.is_new = false;
                    user_configuration.location = UserInputValue::Right(configuration.location.unwrap_or_default().value());
                    user_configuration.devices = configuration.topology.devices.into_iter().map(|device| {
                        let mut configured_clusters = vec![];

                        for cluster in &clusters {
                            for device_id in &cluster.devices {
                                if device.id == *device_id {
                                    configured_clusters.push(cluster.clone());
                                }
                            }
                        }
                        RwSignal::new(UserDeviceConfiguration {
                            id: device.id,
                            name: UserInputValue::Right(device.name.to_string()),
                            interface: Some(device.interface),
                            description: UserInputValue::Right(device.description.unwrap_or_default().to_string()),
                            is_collapsed: true,
                            contained_in_clusters: configured_clusters,
                            tags: device.tags,
                        })
                    }).collect::<Vec<_>>();
                    if let Some(bridge_name) = configuration.network.bridge_name {
                        user_configuration.network.bridge_name = UserInputValue::Right(bridge_name.name());
                    }
                    user_configuration.network.network_interfaces = configuration.network.interfaces.into_iter()
                        .map(|interface| {
                            RwSignal::new(UserNetworkInterface::from(interface))
                        })
                        .collect();
                    for executor in configuration.executors.executors {
                        let ExecutorDescriptor { id, kind, results_url } = executor;

                        let kind = match kind {
                            ExecutorKind::Executable => todo!(),
                            ExecutorKind::Container {
                                engine,
                                name,
                                image,
                                volumes,
                                devices,
                                envs,
                                ports,
                                command,
                                args,
                            } => {
                                let volumes = volumes.into_iter()
                                    .map(|volume| {
                                        RwSignal::new(UserInputValue::Right(volume.to_string()))
                                    })
                                    .collect::<Vec<_>>();
                                let devices = devices.into_iter()
                                    .map(|device| {
                                        RwSignal::new(UserInputValue::Right(device.to_string()))
                                    })
                                    .collect::<Vec<_>>();
                                let envs = envs.into_iter()
                                    .map(|env| {
                                        let (name, value) = env.into();
                                        RwSignal::new(UserContainerEnv {
                                            name: UserInputValue::Right(name),
                                            value: UserInputValue::Right(value)
                                        })
                                    })
                                    .collect::<Vec<_>>();
                                let ports = ports.into_iter()
                                    .map(|port| {
                                        RwSignal::new(UserInputValue::Right(port.to_string()))
                                    })
                                    .collect::<Vec<_>>();
                                let args = args.into_iter()
                                    .map(|arg| {
                                        RwSignal::new(UserInputValue::Right(arg.to_string()))
                                    })
                                    .collect::<Vec<_>>();
                                UserPeerExecutorKind::Container {
                                    engine,
                                    name: UserInputValue::Right(name.into()),
                                    image: UserInputValue::Right(image.to_string()),
                                    volumes,
                                    devices,
                                    envs,
                                    ports,
                                    command: UserInputValue::Right(command.into()),
                                    args,
                                }
                            }
                        };

                        user_configuration.executors.push(
                            RwSignal::new(UserPeerExecutor {
                                id,
                                kind,
                                results_url: UserInputValue::Right(results_url.map(|s| s.to_string()).unwrap_or(String::new())),
                                is_collapsed: true
                            })
                        );
                    }
                });
                if let Ok(state) = carl.peers.get_peer_state(peer_id).await {
                    peer_state.set(state);
                }
            }
        }
    });

    let peer_id_string = create_read_slice(user_peer_descriptor, |config| config.id.to_string());
    let setup_disabled = create_read_slice(user_peer_descriptor, |config| config.is_new);

    let cluster_column = move || {
        let peer = user_peer_descriptor.get();

        let devices_in_peer = peer.devices.into_iter()
            .map(|device| device.get().id)
            .collect::<Vec<_>>();

        let cluster_descriptor = user_peer_descriptor
            .get().devices
            .into_iter()
            .flat_map(|device| device.get().contained_in_clusters)
            .collect::<Vec<_>>();

        let configured_clusters = util::list_configured_clusters_for_peer(devices_in_peer, cluster_descriptor);

        let cluster_view_list: Vec<View<_>> = configured_clusters.into_iter()
            .map(|cluster| {
                let href = format!("/clusters/{}/configure/general", cluster.id);

                view! {
                    <a href=href>{cluster.name.to_string()}</a>
                }
            })
            .collect::<Vec<_>>();

        let amount_clusters = cluster_view_list.len();

        if amount_clusters > 0 {
            let comma_separated_cluster_views = util::view::join_with_comma_spans(cluster_view_list.clone());
            Either::Left(view! {
                <div class="mb-3">"Configured in " {amount_clusters} " clusters: " {comma_separated_cluster_views}</div>
            })
        } else {
            Either::Right(view! {
                <div class="mb-3">"Configured in " {amount_clusters} " clusters"</div>
            })
        }
    };

    let breadcrumbs = Signal::derive(move || {
        let peer_id = peer_id_string.get();
        let active_tab = active_tab.get();
        vec![
            Breadcrumb::new("Dashboard", "/"),
            Breadcrumb::new("Peers", "peers"),
            Breadcrumb::new("Configure Peer", format!("{}/configure/{}", peer_id, active_tab.as_str())),
        ]
    });

    let subtitle = Signal::derive(move || {
        if let UserInputValue::Right(name) = user_peer_descriptor.get().name {
            name
        } else {
            String::new()
        }
    });

    let tabs = Signal::derive(move || {

        let tabs = vec![
            Tab::from_title_and_href(
                String::from("General"),
                TabIdentifier::General.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_general_tab())),

            Tab::from_title_and_href(
                String::from("Network"),
                TabIdentifier::Network.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_network_tab())),

            Tab::from_title_and_href(
                String::from("Devices"),
                TabIdentifier::Devices.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_devices_tab())),

            Tab::from_title_and_href(
                String::from("Executor"),
                TabIdentifier::Executor.as_str().to_owned()
            ).with_is_error(Signal::derive(move || !user_peer_descriptor.read().valid_executor_tab())),

            Tab::from_title_and_href(
                String::from("Setup"),
                TabIdentifier::Setup.as_str().to_owned()
            ).with_is_hidden(setup_disabled),
        ];

        tabs
    });

    view! {
        <BasePageContainer
            title="Configure Peer"
            subtitle=subtitle
            breadcrumbs=breadcrumbs
            controls=view! { <Controls user_peer_descriptor peer_state=peer_state.into() /> }
        >
        <div> {cluster_column} </div>
            <Suspense
                fallback=LoadingSpinner // TODO: Display errors
            >
            { move || Suspend::new(async move {
                peer_descriptor_resource.await;

                view! {
                    <Tabs tabs active_tab=Signal::derive(move || active_tab.get().as_str())>
                        <div class=("is-hidden", move || TabIdentifier::General != active_tab.get())>
                            <GeneralTab user_peer_descriptor />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Network != active_tab.get())>
                            <NetworkTab user_peer_descriptor />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Devices != active_tab.get())>
                            <DevicesTab user_peer_descriptor />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Executor != active_tab.get())>
                            <ExecutorTab user_peer_descriptor />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Setup != active_tab.get())>
                            <SetupTab peer_configuration=user_peer_descriptor.read_only() />
                        </div>
                    </Tabs>
                }
            })}
            </Suspense>
        </BasePageContainer>
    }
}
