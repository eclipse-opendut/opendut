use leptos::*;

use opendut_types::cluster::{IllegalClusterName, ClusterName};

use crate::components::{UserInput, UserInputValue};
use crate::clusters::configurator::types::UserClusterConfiguration;

#[component]
pub fn ClusterNameInput(cluster_configuration: RwSignal<UserClusterConfiguration>) -> impl IntoView {

    let (getter, setter) = create_slice(cluster_configuration,
        |config| {
            Clone::clone(&config.name)
        },
        |config, input| {
            config.name = input;
        }
    );

    let validator = |input: String| {
        match ClusterName::try_from(input) {
            Ok(name) => {
                UserInputValue::Right(name.value())
            }
            Err(cause) => {
                match cause {
                    IllegalClusterName::TooShort { expected, actual, value } => {
                        if actual > 0 {
                            UserInputValue::Both(format!("A cluster name must be at least {} characters long.", expected), value)
                        }
                        else {
                            UserInputValue::Both("Enter a valid cluster name.".to_string(), value)
                        }
                    },
                    IllegalClusterName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A cluster name must be at most {} characters long.", expected), value)
                    },
                    IllegalClusterName::InvalidStartEndCharacter { value } => {
                        UserInputValue::Both("The cluster name starts/ends with an invalid character. \
                        Valid characters are a-z, A-Z and 0-9.".to_string(), value)
                    }
                    IllegalClusterName::InvalidCharacter { value } => {
                        UserInputValue::Both("The cluster name contains invalid characters. \
                        Valid characters are a-z, A-Z, 0-9 and _-".to_string(), value)
                    },
                }
            }
        }
    };

    view! {
        <UserInput
            getter=getter
            setter=setter
            label="Cluster Name"
            placeholder="AwesomeCluster"
            validator=validator
        />
    }
}
