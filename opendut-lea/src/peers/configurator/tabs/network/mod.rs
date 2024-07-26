use std::rc::Rc;

use leptos::{component, create_action, create_read_slice, create_rw_signal, create_slice, IntoView, RwSignal, SignalGetUntracked, SignalWith, SignalWithUntracked, view};
use opendut_types::util::net::NetworkInterfaceId;
use crate::components::{Toast, use_toaster, UserInputValue};
use crate::peers::configurator::tabs::network::network_interface_input::NetworkInterfaceInput;
use crate::peers::configurator::tabs::network::bridge_name_input::BridgeNameInput;
use crate::peers::configurator::types::{UserPeerConfiguration, UserNetworkInterface};

mod network_interface_input;
mod bridge_name_input;

#[component]
pub fn NetworkTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {
    let toaster = use_toaster();
    let toaster = Rc::clone(&toaster);

    let (interfaces_getter, interfaces_setter) = create_slice(peer_configuration,
         |peer_configuration| {
             Clone::clone(&peer_configuration.network.network_interfaces)
         },
         |peer_configuration, value| {
             peer_configuration.network.network_interfaces = value
         }
    );

    let devices = create_read_slice(peer_configuration,
         |peer_configuration| {
             Clone::clone(&peer_configuration.devices)
         }
    );

    let user_input_string = move |user_input| {
        match user_input {
            UserInputValue::Left(_) => String::new(),
            UserInputValue::Right(value) => value.to_owned(),
            UserInputValue::Both(_, value) => value.to_owned(),
        }
    };

    let interface_name = move | user_network_interface: Option<UserNetworkInterface> | {
        match user_network_interface {
            None => { String::new() }
            Some(interface) => { interface.name.name() }
        }
    };

    let device_interfaces = move || {
        devices.with(|devices| {
            devices.iter()
                .cloned()
                .map(|device_configuration| {
                    interface_name(device_configuration.get_untracked().interface)
                })
                .collect::<Vec<_>>()
        })
    };

    let deletion_failed_action = create_action(move |interface_name_to_delete: &String| {
        let toaster = Rc::clone(&toaster);
        let devices_with_interface = devices.get_untracked().into_iter()
            .filter(|device| {
                let device_interface_name = interface_name(device.get_untracked().interface);
                &device_interface_name == interface_name_to_delete
            })
            .map(|device| {
                let device = device.get_untracked();
                let name = user_input_string(device.name);
                if name.is_empty() {
                    device.id.to_string()
                } else {
                    name
                }
            })
            .collect::<Vec<_>>();
        async move {
            toaster.toast(
                Toast::builder()
                    .simple(format!("Network interface could not be deleted due to it being used in following devices: {}", devices_with_interface.join(", ")))
                    .error(),
            )
        }
    });

    let interface_name_list = move || {
        interfaces_getter.with(|interface_names| {
            interface_names.iter()
                .cloned()
                .map(|interface| {
                    let name = interface.get_untracked().name;
                    let configuration_name = interface.get_untracked().configuration.display_name();
                    view! {
                        <span class="tag is-info is-primary">
                            { &name.name() } " (" { configuration_name } ")"
                                <button class="delete" type="button"
                                    on:click=move |_| {
                                        if device_interfaces().contains(&name.name()) {
                                            deletion_failed_action.dispatch(name.name().clone());
                                        } else {
                                            let remaining_interfaces = interfaces_getter.with_untracked(|interfaces| {
                                                interfaces.iter()
                                                    .filter(|&interface_name| name != interface_name.get_untracked().name)
                                                    .cloned()
                                                    .collect::<Vec<_>>()
                                            });
                                            interfaces_setter.set(remaining_interfaces);
                                        }
                                    }
                                ></button>
                        </span>
                    }
                })
                .collect::<Vec<_>>()
        })
    };

    view! {
        <div class="box">
            <h5 class="title is-5">Network Interfaces</h5>
            <NetworkInterfaceInput
                interfaces = interfaces_getter
                on_action = move |name, configuration| {
                    let mut interfaces = interfaces_getter.get_untracked();
                    let user_peer_network = create_rw_signal(
                        UserNetworkInterface {
                            id: NetworkInterfaceId::random(),
                            name,
                            configuration
                        }
                    );
                    interfaces.push(user_peer_network);
                    interfaces_setter.set(interfaces);
                }
            />
            <label class="label">Configured Network Interfaces</label>
            <div class="tags are-medium">
                { interface_name_list }
            </div>
        </div>
        <div class="box">
            <BridgeNameInput
                peer_configuration=peer_configuration
            />
        </div>
    }
}
