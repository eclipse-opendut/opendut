use std::collections::HashSet;
use serde::Serialize;
use crate::cluster::ClusterAssignment;

pub mod api;
pub use crate::peer::configuration::api::*;

pub mod parameter;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct OldPeerConfiguration {
    pub cluster_assignment: Option<ClusterAssignment>,
    // Please add new fields into PeerConfiguration instead.
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PeerConfiguration {
    pub device_interfaces: Vec<Parameter<parameter::DeviceInterface>>,
    pub ethernet_bridges: Vec<Parameter<parameter::EthernetBridge>>,
    pub executors: Vec<Parameter<parameter::Executor>>,
    pub gre_interfaces: Vec<Parameter<parameter::GreInterfaceConfig>>,
    pub joined_interfaces: Vec<Parameter<parameter::InterfaceJoinConfig>>,
    pub remote_peer_connection_checks: Vec<Parameter<parameter::RemotePeerConnectionCheck>>,
    //TODO migrate more parameters
}
impl PeerConfiguration {
    /// Set all parameters of a type to be present/absent.
    /// Parameters that were previously in the PeerConfiguration,
    /// but aren't in the new list, will be set to absent.
    pub fn set_all_present<T: ParameterValue>(&mut self, values: impl IntoIterator<Item=T>) {

        let new_present_parameters = values.into_iter()
            .map(|value| Self::create_parameter(value, ParameterTarget::Present))
            .collect::<HashSet<_>>();

        let previous_parameters = T::peer_configuration_field(self)
            .iter_mut()
            .map(|parameter_ref| parameter_ref.to_owned())
            .collect::<HashSet<_>>();

        let parameters_to_set_absent = previous_parameters.difference(&new_present_parameters);

        for parameter in parameters_to_set_absent {
            let absent_parameter = Parameter {
                target: ParameterTarget::Absent,
                ..parameter.clone()
            };
            self.set_parameter(absent_parameter);
        }

        for parameter in new_present_parameters {
            self.set_parameter(parameter);
        }
    }

    /// Set an individual parameter to be present/absent
    pub fn set<T: ParameterValue>(&mut self, value: T, target: ParameterTarget) {
        let parameter = Self::create_parameter(value, target);

        self.set_parameter(parameter);
    }


    fn set_parameter<T: ParameterValue>(&mut self, parameter: Parameter<T>) {
        let parameters = T::peer_configuration_field(self);

        parameters.retain(|existing_parameter| {
            existing_parameter.id != parameter.id
        });

        parameters.push(parameter);
    }

    fn create_parameter<T: ParameterValue>(value: T, target: ParameterTarget) -> Parameter<T> {
        Parameter {
            id: value.parameter_identifier(),
            dependencies: vec![], //TODO
            target,
            value,
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind, ResultsUrl};
    use crate::util::net::NetworkInterfaceName;
    use super::*;

    mod set {
        use super::*;

        #[test]
        fn should_replace_a_previous_parameter_when_it_is_set_another_time() -> anyhow::Result<()> {

            let parameter_value = parameter::EthernetBridge { name: NetworkInterfaceName::try_from("br-opendut")? };

            let mut testee = PeerConfiguration::default();
            testee.set(parameter_value.clone(), ParameterTarget::Present);


            testee.set(parameter_value.clone(), ParameterTarget::Present);
            assert_eq!(testee.ethernet_bridges.len(), 1);

            testee.set(parameter_value.clone(), ParameterTarget::Absent);
            assert_eq!(testee.ethernet_bridges.len(), 1);
            assert_eq!(testee.ethernet_bridges[0].target, ParameterTarget::Absent);

            Ok(())
        }


        #[test]
        fn should_update_the_value_of_a_parameter() -> anyhow::Result<()> {

            let parameter_value = parameter::Executor {
                descriptor: ExecutorDescriptor {
                    id: ExecutorId::random(),
                    kind: ExecutorKind::Executable,
                    results_url: Some(ResultsUrl::try_from("https://example.com")?),
                }
            };

            let mut testee = PeerConfiguration::default();
            testee.set(parameter_value.clone(), ParameterTarget::Present);


            let expected = None;
            let parameter_value = parameter::Executor {
                descriptor: ExecutorDescriptor {
                    results_url: expected.clone(),
                    ..parameter_value.descriptor
                }
            };

            testee.set(parameter_value, ParameterTarget::Present);
            assert_eq!(testee.executors.len(), 1);
            assert_eq!(testee.executors[0].value.descriptor.results_url, expected);

            Ok(())
        }
    }

    mod set_all_present {
        use super::*;
        use googletest::prelude::*;

        #[test]
        fn should_mark_obsolete_parameters_as_absent_and_retain_or_set_other_parameters_as_present() -> anyhow::Result<()> {

            fn parameter_value(id: &str) -> parameter::EthernetBridge {
                parameter::EthernetBridge { name: NetworkInterfaceName::try_from(id).unwrap() }
            }

            let present_then_absent = parameter_value("1");
            let present_then_present = parameter_value("2");
            let new_present = parameter_value("3");
            let absent_then_present = parameter_value("4");

            let mut testee = PeerConfiguration::default();

            testee.set(present_then_absent.clone(), ParameterTarget::Present);
            testee.set(present_then_present.clone(), ParameterTarget::Present);
            testee.set(absent_then_present.clone(), ParameterTarget::Absent);

            testee.set_all_present([
                present_then_present.clone(),
                new_present.clone(),
                absent_then_present.clone()
            ]);

            assert_that!(testee.ethernet_bridges, unordered_elements_are![
                matches_pattern!(Parameter {
                    value: eq(&present_then_absent),
                    target: eq(&ParameterTarget::Absent),
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&present_then_present),
                    target: eq(&ParameterTarget::Present),
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&new_present),
                    target: eq(&ParameterTarget::Present),
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&absent_then_present),
                    target: eq(&ParameterTarget::Present),
                    ..
                }),
            ]);

            Ok(())
        }
    }
}
