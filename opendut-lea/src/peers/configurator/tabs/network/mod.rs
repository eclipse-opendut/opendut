use std::sync::Arc;

use leptos::prelude::*;

use opendut_types::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceId};

use crate::components::{Toast, use_toaster, UserInputValue};
use crate::peers::configurator::tabs::network::bridge_name_input::BridgeNameInput;
use crate::peers::configurator::tabs::network::network_interface_input::NetworkInterfaceInput;
use crate::peers::configurator::types::{UserNetworkInterface, UserPeerConfiguration};

mod network_interface_input;
mod bridge_name_input;

#[component]
pub fn NetworkTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {

    let (interfaces_getter, interfaces_setter) = create_slice(peer_configuration,
         |peer_configuration| {
             Clone::clone(&peer_configuration.network.network_interfaces)
         },
         |peer_configuration, mut value: Vec<RwSignal<UserNetworkInterface>>| {
             value.sort_by(|user_network_interface_left, user_network_interface_right| {
                 user_network_interface_left.get().configuration.display_name().cmp(&user_network_interface_right.get().configuration.display_name())
             });
             peer_configuration.network.network_interfaces = value;
         }
    );
    
    let interface_name_list = move || {
        interfaces_getter.with(|interface_names| {
            interface_names.iter()
                .cloned()
                .map(|interface| {
                    view! {
                        <Row
                            network_interface=interface
                            peer_configuration=peer_configuration
                        />
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
                    let user_peer_network = RwSignal::new(
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
                <table class="table is-hoverable is-fullwidth">
                    <thead>
                        <tr>
                            <th class="is-narrow">"Interface Name"</th>
                            <th class="is-narrow">"Type"</th>
                            <th class="is-narrow">"Bitrate"</th>
                            <th class="is-narrow">"Sample Point"</th>
                            <th class="is-narrow">"CAN FD"</th>
                            <th class="is-narrow">"Data Bitrate"</th>
                            <th class="is-narrow">"Data Sample Point"</th>
                            <th class="is-narrow"></th>
                        </tr>
                    </thead>
                    <tbody>
                        { interface_name_list }
                    </tbody>
                </table>
            </div>
        </div>
        <div class="box">
            <BridgeNameInput
                peer_configuration=peer_configuration
            />
        </div>
    }
}

#[component]
fn Row(
    network_interface: RwSignal<UserNetworkInterface>,
    peer_configuration: RwSignal<UserPeerConfiguration>,
) -> impl IntoView {

    let toaster = use_toaster();

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

    let interfaces_used_by_a_device = move || {
        devices.with(|devices| {
            devices.iter()
                .cloned()
                .flat_map(|device_configuration| device_configuration.get_untracked().interface)
                .collect::<Vec<_>>()
        })
    };

    let deletion_failed_action = Action::new(move |interface_to_delete: &NetworkInterfaceId| {
        let toaster = Arc::clone(&toaster);
        let devices_with_interface = devices.get_untracked().into_iter()
            .filter(|device| {
                device.get_untracked().interface == Some(*interface_to_delete)
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
    
    let user_network_interface = network_interface.get();
    let network_interface_name = Signal::derive(
        move || user_network_interface.name.name()
    );
    
    let network_configuration_id = user_network_interface.id;
    
    let network_interface_function = move || {
        match &user_network_interface.configuration.inner {
            NetworkInterfaceConfiguration::Ethernet => {
                (
                    network_configuration_id,
                    user_network_interface.configuration.display_name(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string(),
                    "-".to_string()
                )
            }
            NetworkInterfaceConfiguration::Can { sample_point, data_sample_point, data_bitrate, bitrate, fd } => {
                (
                    network_configuration_id,
                    user_network_interface.configuration.display_name(),
                    bitrate.to_string(),
                    sample_point.to_string(),
                    fd.to_string(),
                    data_bitrate.to_string(),
                    data_sample_point.to_string()
                )
            }
        }
    };
    
    let (id, network_type, bitrate, sample_point, fd, data_bitrate, data_sample_point) = network_interface_function();
    
    view! {
        <tr>
            <td class="is-vcentered">
                { network_interface_name }
            </td>
            <td class="is-vcentered">
                { network_type }
            </td>
            <td class="is-vcentered">
                { bitrate }
            </td>
            <td class="is-vcentered">
                { sample_point }
            </td>
            <td class="is-vcentered">
                { fd }
            </td>
            <td class="is-vcentered">
                { data_bitrate }
            </td>
            <td class="is-vcentered">
                { data_sample_point }
            </td>
            <td class="is-vcentered">
                <button class="delete button is-danger"
                    on:click=move |_| {
                        if interfaces_used_by_a_device().contains(&id) {
                            deletion_failed_action.dispatch(id);
                        } else {
                            let remaining_interfaces = interfaces_getter.with_untracked(|interfaces| {
                                interfaces.iter()
                                    .filter(|&interface| id != interface.get_untracked().id)
                                    .cloned()
                                    .collect::<Vec<_>>()
                            });
                            interfaces_setter.set(remaining_interfaces);
                        }
                    }
                ></button>
            </td>
        </tr>
    }
}
