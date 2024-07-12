use crate::components::{UserInput, UserInputValue};
use crate::peers::configurator::types::UserPeerConfiguration;
use leptos::{component, create_slice, view, IntoView, RwSignal};
use opendut_types::util::net::{NetworkInterfaceName, NetworkInterfaceNameError};

#[component]
pub fn BridgeNameInput(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {
    let (getter, setter) = create_slice(
        peer_configuration,
        |config| Clone::clone(&config.network.bridge_name),
        |config, input| {
            config.network.bridge_name = input;
        },
    );

    let validator = |input: String| {
        match NetworkInterfaceName::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    NetworkInterfaceNameError::Empty => {
                        UserInputValue::Right(String::new())
                    }
                    NetworkInterfaceNameError::TooLong { value, max } => {
                        UserInputValue::Both(format!("A bridge name must be at most {} characters long.", max), value)
                    },
                }
            }
        }
    };

    view! {
        <div>
            <UserInput
                getter=getter
                setter=setter
                label="Custom Bridge Name"
                placeholder="br-opendut"
                validator=validator
            />
            <div class="notification is-warning">
                <div class="columns is-mobile is-vcentered">
                    <div class="column is-narrow">
                        <i class="fa-solid fa-triangle-exclamation fa-2xl"></i>
                    </div>
                    <div class="column">
                        <p>"Bridges with custom names are not automatically removed and need to be removed manually. Not removing the bridge could lead to network traffic being misdirected!"</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
