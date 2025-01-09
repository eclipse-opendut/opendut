use crate::components::{UserInput, UserInputValue};
use crate::peers::configurator::types::UserPeerConfiguration;
use leptos::prelude::*;
use opendut_types::peer::{IllegalLocation, PeerLocation};

#[component]
pub fn PeerLocationInput(peer_configuration: RwSignal<UserPeerConfiguration>) -> impl IntoView {
    let (getter, setter) = create_slice(
        peer_configuration,
        |config| Clone::clone(&config.location),
        |config, input| {
            config.location = input;
        },
    );

    let validator = |input: String| {
        match PeerLocation::try_from(input.clone()) {
            Ok(_) => {
                UserInputValue::Right(input)
            }
            Err(cause) => {
                match cause {
                    IllegalLocation::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A peer location must be at most {} characters long.", expected), value)
                    },
                    IllegalLocation::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The peer name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z, 0-9 and ()".to_string(), value)
                    },
                    IllegalLocation::InvalidCharacter { value } => {
                        UserInputValue::Both("The peer location contains invalid characters. \
                        Valid characters are a-z, A-Z, 0-9 and -_/.,() and whitespace".to_string(), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Location"
            placeholder="Ulm, Germany"
            validator=validator
        />
    }
}
