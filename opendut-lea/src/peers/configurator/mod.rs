use leptos::*;
use leptos_router::use_params_map;

use opendut_types::peer::PeerId;

use crate::app::{ExpectGlobals, use_app_globals};
use crate::components::{BasePageContainer, Breadcrumb, Initialized, UserInputError, UserInputValue};
use crate::components::use_active_tab;
use crate::peers::configurator::components::Controls;
use crate::peers::configurator::tabs::{DevicesTab, ExecutorTab, GeneralTab, NetworkTab, SetupTab, TabIdentifier};
use crate::peers::configurator::types::{UserDeviceConfiguration, UserPeerConfiguration, UserPeerNetworkInterface, UserPeerExecutor, UserContainerEnv};
use crate::routing::{navigate_to, WellKnownRoutes};

mod components;
mod tabs;
mod types;

#[component(transparent)]
pub fn PeerConfigurator() -> impl IntoView {

    #[component]
    fn inner() -> impl IntoView {

        let globals = use_app_globals();
        let params = use_params_map();

        let active_tab = use_active_tab::<TabIdentifier>();

        let (peer_configuration, peer_configuration_resource, is_valid_peer_configuration) = {
            let peer_id = {
                let peer_id = params.with_untracked(|params| {
                    params.get("id").and_then(|id| PeerId::try_from(id.as_str()).ok())
                });
                match peer_id {
                    None => {
                        navigate_to(WellKnownRoutes::ErrorPage {
                            title: String::from("Invalid PeerId"),
                            text: String::from("Could not parse the provided value as PeerId!"),
                            details: None,
                        });
                        PeerId::random()
                    }
                    Some(peer_id) => {
                        peer_id
                    }
                }
            };

            let peer_configuration = create_rw_signal(UserPeerConfiguration {
                id: peer_id,
                name: UserInputValue::Left(UserInputError::from("Enter a valid peer name.")),
                location: UserInputValue::Right(String::from("")),
                devices: Vec::new(),
                network_interfaces: Vec::new(),
                is_new: true,
                executors: Vec::new(),
            });

            let peer_configuration_resource = create_local_resource(|| {}, move |_| {
                let mut carl = globals.expect_client();
                async move {
                    if let Ok(configuration) = carl.peers.get_peer_descriptor(peer_id).await {
                        peer_configuration.update(|user_configuration| {
                            user_configuration.name = UserInputValue::Right(configuration.name.value());
                            user_configuration.is_new = false;
                            user_configuration.location = UserInputValue::Right(configuration.location.unwrap_or_default().value());
                            user_configuration.devices = configuration.topology.devices.into_iter().map(|device| {
                                create_rw_signal(UserDeviceConfiguration {
                                    id: device.id,
                                    name: UserInputValue::Right(device.name.to_string()),
                                    interface: UserInputValue::Right(device.interface.name.name()),
                                    description: UserInputValue::Right(device.description.unwrap_or_default().to_string()),
                                    is_collapsed: true
                                })
                            }).collect::<Vec<_>>();
                            user_configuration.network_interfaces = configuration.network_configuration.interfaces.into_iter()
                                .map(|interface| {
                                    create_rw_signal(UserPeerNetworkInterface {
                                        name: interface.name
                                    })
                                })
                                .collect();
                            for executor in configuration.executors.executors {
                                if let opendut_types::peer::executor::ExecutorDescriptor::Container {
                                    engine,
                                    name,
                                    image,
                                    volumes,
                                    devices,
                                    envs,
                                    ports,
                                    command,
                                    args
                                } = executor {
                                    let volumes = volumes.into_iter()
                                        .map(|volume| {
                                            create_rw_signal(UserInputValue::Right(volume.to_string()))
                                        })
                                        .collect::<Vec<_>>();
                                    let devices = devices.into_iter()
                                        .map(|device| {
                                            create_rw_signal(UserInputValue::Right(device.to_string()))
                                        })
                                        .collect::<Vec<_>>();
                                    let envs = envs.into_iter()
                                        .map(|env| {
                                            let (name, value) = env.into();
                                            create_rw_signal(UserContainerEnv {
                                                name: UserInputValue::Right(name),
                                                value: UserInputValue::Right(value)
                                            })
                                        })
                                        .collect::<Vec<_>>();
                                    let ports = ports.into_iter()
                                        .map(|port| {
                                            create_rw_signal(UserInputValue::Right(port.to_string()))
                                        })
                                        .collect::<Vec<_>>();
                                    let args = args.into_iter()
                                        .map(|arg| {
                                            create_rw_signal(UserInputValue::Right(arg.to_string()))
                                        })
                                        .collect::<Vec<_>>();
                                    user_configuration.executors.push(
                                        create_rw_signal(UserPeerExecutor::Container {
                                            engine,
                                            name: UserInputValue::Right(name.into()),
                                            image: UserInputValue::Right(image.to_string()),
                                            volumes,
                                            devices,
                                            envs,
                                            ports,
                                            command: UserInputValue::Right(command.into()),
                                            args,
                                            is_collapsed: true
                                        }));
                                }
                            }
                        });
                    }
                }
            });

            let is_valid_peer_configuration = create_memo(move |_| {
                peer_configuration.with(|peer_configuration| {
                    peer_configuration.name.is_right()
                        && peer_configuration.location.is_right()
                    && peer_configuration.devices.iter().all(|device_configuration| {
                        device_configuration.with(|device_configuration| {
                            device_configuration.name.is_right()
                            && device_configuration.interface.is_right()
                        })
                    })
                    && peer_configuration.executors.iter().all(|executor| {
                        executor.with(|executor| {
                            match executor {
                                UserPeerExecutor::Container {
                                    name, 
                                    image, 
                                    volumes, 
                                    devices, 
                                    envs, 
                                    ports, 
                                    command, 
                                    args, 
                                    ..
                                } => {
                                    name.is_right() 
                                        && image.is_right() 
                                        && volumes.iter().all(|volume| volume.with(|volume| volume.is_right()))
                                        && devices.iter().all(|device| device.with(|device| device.is_right()))
                                        && envs.iter().all(|env| env.with(|env| env.name.is_right()))
                                        && ports.iter().all(|port| port.with(|port| port.is_right()))
                                        && command.is_right()
                                        && args.iter().all(|arg| arg.with(|arg| arg.is_right()))
                                }
                            }
                        })
                    })
                })
            });

            (peer_configuration, peer_configuration_resource, is_valid_peer_configuration)
        };

        let peer_id_string = create_read_slice(peer_configuration, |config| config.id.to_string());
        let setup_disabled = create_read_slice(peer_configuration, |config| config.is_new);

        let setup_tab_classes = move || {
            let mut classes = Vec::<&'static str>::new();
            if TabIdentifier::Setup == active_tab.get() {
                classes.push("is-active");
            }
            if setup_disabled.get() {
                classes.push("is-hidden");
            }
            classes.join(" ")
        };

        let breadcrumbs = {
            let active_tab = Clone::clone(&active_tab);
            MaybeSignal::derive(move || {
                let peer_id = peer_id_string.get();
                let active_tab = active_tab.get();
                vec![
                    Breadcrumb::new("Dashboard", "/"),
                    Breadcrumb::new("Peers", "peers"),
                    Breadcrumb::new(Clone::clone(&peer_id), format!("{}/configure/{}", peer_id, active_tab.as_str())),
                ]
            })
        };

        view! {
            <BasePageContainer
                title="Configure Peer"
                breadcrumbs=breadcrumbs
                controls=view! { <Controls configuration=peer_configuration is_valid_peer_configuration=is_valid_peer_configuration.into() /> }
            >
                <Show
                    when=move || !peer_configuration_resource.loading().get() // TODO: Check for errors
                    fallback=move || view! { <p><i class="fa-solid fa-circle-notch fa-spin"></i></p> } // TODO: Display errors
                >
                    <div class="tabs">
                        <ul>
                            <li class=("is-active", move || TabIdentifier::General == active_tab.get())>
                                <a href={ TabIdentifier::General.as_str() }>
                                    <span>General</span>
                                    // An icon could indicate a misconfiguration on a tab
                                    // <span class="icon is-small has-text-danger"><i class="fa-solid fa-circle-exclamation" aria-hidden="true"></i></span>
                                </a>
                            </li>
                            <li class=("is-active", move || TabIdentifier::Network == active_tab.get())>
                                <a href={ TabIdentifier::Network.as_str() }>Network</a>
                            </li>
                            <li class=("is-active", move || TabIdentifier::Executor == active_tab.get())>
                                <a href={ TabIdentifier::Executor.as_str() }>Executor</a>
                            </li>
                            <li class=("is-active", move || TabIdentifier::Devices == active_tab.get())>
                                <a href={ TabIdentifier::Devices.as_str() }>Devices</a>
                            </li>
                            <li class=setup_tab_classes>
                                <a href={ TabIdentifier::Setup.as_str() }>Setup</a>
                            </li>
                        </ul>
                    </div>
                    <div class="container">
                        <div class=("is-hidden", move || TabIdentifier::General != active_tab.get())>
                            <GeneralTab peer_configuration=peer_configuration />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Network != active_tab.get())>
                            <NetworkTab peer_configuration=peer_configuration />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Executor != active_tab.get())>
                            <ExecutorTab peer_configuration=peer_configuration />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Devices != active_tab.get())>
                            <DevicesTab peer_configuration=peer_configuration />
                        </div>
                        <div class=("is-hidden", move || TabIdentifier::Setup != active_tab.get())>
                            <SetupTab peer_configuration=peer_configuration.read_only() />
                        </div>
                    </div>
                </Show>
            </BasePageContainer>
        }
    }

    view! {
        <Initialized>
            <Inner />
        </Initialized>
    }
}
