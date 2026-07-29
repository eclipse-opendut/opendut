use leptos::prelude::*;

use network_interface_list::NetworkInterfaceList;
use opendut_model::util::net::NetworkInterfaceId;

use crate::peers::configurator::tabs::network::bridge_name_input::BridgeNameInput;
use crate::peers::configurator::tabs::network::network_interface_input::NetworkInterfaceInput;
use crate::peers::configurator::types::network::UserNetworkInterface;
use crate::peers::configurator::types::UserPeerDescriptor;

mod bridge_name_input;
mod network_interface_input;
mod network_interface_list;


#[component]
pub fn NetworkTab(user_peer_descriptor: RwSignal<UserPeerDescriptor>) -> impl IntoView {

    let (interfaces, set_interfaces) = create_slice(user_peer_descriptor,
         |user_peer_descriptor| {
             Clone::clone(&user_peer_descriptor.network.network_interfaces)
         },
         |user_peer_descriptor, mut value: Vec<RwSignal<UserNetworkInterface>>| {
             value.sort_by(|user_network_interface_left, user_network_interface_right| {
                 user_network_interface_left.get().configuration.display_name()
                    .cmp(&user_network_interface_right.get().configuration.display_name())
             });
             user_peer_descriptor.network.network_interfaces = value;
         }
    );

    view! {
        <div class="box">
            <h5 class="title is-5">Network Interfaces</h5>
            <NetworkInterfaceInput
                interfaces
                on_action = move |name, configuration| {
                    let mut interfaces = interfaces.get_untracked();
                    let interface = RwSignal::new(
                        UserNetworkInterface {
                            id: NetworkInterfaceId::random(),
                            name,
                            configuration
                        }
                    );
                    interfaces.push(interface);
                    set_interfaces.set(interfaces);
                }
            />
            <label class="label">Configured Network Interfaces</label>
            <NetworkInterfaceList interfaces user_peer_descriptor />
        </div>
        <div class="box">
            <BridgeNameInput user_peer_descriptor />
        </div>
    }
}
