use leptos::*;

use opendut_types::peer::{IllegalPeerName, PeerName};

use crate::components::{UserInput, UserInputValue};
use crate::peers::configurator::types::UserPeerConfiguration;

#[component]
pub fn PeerNameInput(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(peer_configuration,
        |config| {
            Clone::clone(&config.name)
        },
        |config, input| {
            config.name = input;
        }
    );

    let validator = |input: String| {
        match PeerName::try_from(input) {
            Ok(name) => {
                UserInputValue::Right(name.value())
            }
            Err(cause) => {
                match cause {
                    IllegalPeerName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A peer name must be at least {} characters long.", expected), value)
                        }
                        else {
                            UserInputValue::Both(format!("Enter a valid peer name."), value)
                        }
                    },
                    IllegalPeerName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A peer name must be at most {} characters long.", expected), value)
                    },
                    IllegalPeerName::InvalidCharacter { value } => {
                        UserInputValue::Both(format!("The cluster name contains invalid characters."), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Peer Name"
            placeholder="MyAwesomePeer"
            validator=validator
        />
    }
}
