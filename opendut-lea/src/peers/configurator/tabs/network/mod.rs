use leptos::prelude::*;

use network_interface_list::NetworkInterfaceList;
use opendut_model::util::net::NetworkInterfaceId;

use crate::peers::configurator::tabs::network::bridge_name_input::BridgeNameInput;
use crate::peers::configurator::tabs::network::network_interface_input::NetworkInterfaceInput;
use crate::peers::configurator::types::network::UserNetworkInterface;
use crate::peers::configurator::types::UserPeerConfiguration;

mod bridge_name_input;
mod network_interface_input;
mod network_interface_list;


#[component]
pub fn NetworkTab(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {

    let (interfaces, set_interfaces) = create_slice(peer_configuration,
         |peer_configuration| {
             Clone::clone(&peer_configuration.network.network_interfaces)
         },
         |peer_configuration, mut value: Vec<RwSignal<UserNetworkInterface>>| {
             value.sort_by(|user_network_interface_left, user_network_interface_right| {
                 user_network_interface_left.get().configuration.display_name()
                    .cmp(&user_network_interface_right.get().configuration.display_name())
             });
             peer_configuration.network.network_interfaces = value;
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
            <NetworkInterfaceList interfaces peer_configuration />
        </div>
        <div class="box">
            <BridgeNameInput peer_configuration />
        </div>
    }
}
