use leptos::prelude::*;
use uuid::Uuid;
use opendut_lea_components::{SelectionOption, UserInputValue, UserSelect};
use opendut_model::util::net::NetworkInterfaceId;
use crate::peers::configurator::types::devices::UserDeviceConfiguration;
use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn DeviceInterfaceInput(
    peer_configuration: RwSignal<UserPeerConfiguration>,
    device_configuration: RwSignal<UserDeviceConfiguration>,
) -> impl IntoView {
    const INITIAL_OPTION: &str = "Select interface";

    let peer_network_interfaces = create_read_slice(
        peer_configuration,
        |peer_configuration| {
            Clone::clone(&peer_configuration.network.network_interfaces)
        },
    );

    let (getter, setter) = create_slice(
        device_configuration,
        |device_configuration| {
            match device_configuration.interface.as_ref() {
                Some(interface_id) => {
                    UserInputValue::Right(interface_id.to_string())
                }
                None => {
                    UserInputValue::Left(INITIAL_OPTION.to_owned())
                }
            }
        },
        |device_configuration, input| {
            device_configuration.interface = match input {
                UserInputValue::Right(value)
                | UserInputValue::Both(_, value) => {
                    let uuid = Uuid::parse_str(&value).expect(
                        "Should be a valid UUID, which we passed in as option-value.",
                    );

                    Some(NetworkInterfaceId::from(uuid))
                }
                UserInputValue::Left(_) => None,
            };
        },
    );

    let options = Signal::derive(move || {
        peer_network_interfaces.with(|interfaces| {
            interfaces.iter()
                .map(|interface| {
                    let interface = interface.get_untracked();

                    SelectionOption {
                        display_name: format!(
                            "{} ({})",
                            interface.name.name(),
                            interface.configuration.display_name(),
                        ),
                        value: interface.id.to_string(),
                    }
                })
                .collect()
        })
    });

    let initial_option = INITIAL_OPTION.to_owned();

    view! {
        <UserSelect
            options
            initial_option
            getter
            setter
            label="Interface"
        />
    }
}
