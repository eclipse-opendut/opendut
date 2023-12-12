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
                            UserInputValue::Both(format!("Enter a valid cluster name."), value)
                        }
                    },
                    IllegalClusterName::TooLong { expected, value, .. } => {
                        UserInputValue::Both(format!("A cluster name must be at most {} characters long.", expected), value)
                    },
                    IllegalClusterName::InvalidCharacter { value } => {
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
            label="Cluster Name"
            placeholder="AwesomeCluster"
            validator=validator
        />
    }
}
