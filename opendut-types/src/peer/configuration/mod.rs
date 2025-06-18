use std::collections::HashSet;
use std::ops::{Deref, DerefMut};
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
    pub device_interfaces: ParameterField<parameter::DeviceInterface>,
    pub ethernet_bridges: ParameterField<parameter::EthernetBridge>,
    pub executors: ParameterField<parameter::Executor>,
    pub gre_interfaces: ParameterField<parameter::GreInterfaceConfig>,
    pub joined_interfaces: ParameterField<parameter::InterfaceJoinConfig>,
    pub remote_peer_connection_checks: ParameterField<parameter::RemotePeerConnectionCheck>,
    //TODO migrate more parameters
}
impl<V: ParameterValue> ParameterField<V> {
    /// Set all parameters of a type to be present.
    /// Parameters that were previously in the PeerConfiguration,
    /// but aren't in the new list, will be set to absent.
    pub fn set_all_present(&mut self, values: impl IntoIterator<Item=V>) {

        let new_present_parameters = values.into_iter()
            .map(|value| Self::create_parameter(value, ParameterTarget::Present))
            .collect::<HashSet<_>>();

        let previous_parameters = self.iter_mut()
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

    /// Set all parameters of a type to be absent.
    pub fn set_all_absent(&mut self) {
        let parameters_to_set_absent = self.clone();

        for parameter in parameters_to_set_absent {
            let absent_parameter = Parameter {
                target: ParameterTarget::Absent,
                ..parameter.clone()
            };
            self.set_parameter(absent_parameter);
        }
    }

    /// Set an individual parameter to be present/absent
    pub fn set(&mut self, value: V, target: ParameterTarget) {
        let parameter = Self::create_parameter(value, target);

        self.set_parameter(parameter);
    }


    fn set_parameter(&mut self, parameter: Parameter<V>) {
        let parameters = self;

        parameters.retain(|existing_parameter| {
            existing_parameter.id != parameter.id
        });

        parameters.push(parameter);
    }

    fn create_parameter(value: V, target: ParameterTarget) -> Parameter<V> {
        Parameter {
            id: value.parameter_identifier(),
            dependencies: vec![], //TODO
            target,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ParameterField<V: ParameterValue> {
    pub values: Vec<Parameter<V>>,
}
impl<V: ParameterValue> Default for ParameterField<V> {
    fn default() -> Self {
        Self { values: vec![] }
    }
}
impl<V: ParameterValue> Deref for ParameterField<V> {
    type Target = Vec<Parameter<V>>;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}
impl<V: ParameterValue> DerefMut for ParameterField<V> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.values
    }
}
impl<V: ParameterValue> IntoIterator for ParameterField<V> {
    type Item = Parameter<V>;
    type IntoIter = <Vec<Parameter<V>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.values.into_iter()
    }
}
impl<V: ParameterValue> FromIterator<Parameter<V>> for ParameterField<V> {
    fn from_iter<T: IntoIterator<Item=Parameter<V>>>(iter: T) -> Self {
        Self {
            values: iter.into_iter().collect(),
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind, ResultsUrl};
    use crate::util::net::NetworkInterfaceName;
    use super::*;
    use googletest::prelude::*;

    mod set {
        use super::*;

        #[test]
        fn should_replace_a_previous_parameter_when_it_is_set_another_time() -> anyhow::Result<()> {

            let parameter_value = parameter::EthernetBridge { name: NetworkInterfaceName::try_from("br-opendut")? };

            let mut testee = PeerConfiguration::default();
            testee.ethernet_bridges.set(parameter_value.clone(), ParameterTarget::Present);


            testee.ethernet_bridges.set(parameter_value.clone(), ParameterTarget::Present);
            assert_eq!(testee.ethernet_bridges.len(), 1);

            testee.ethernet_bridges.set(parameter_value.clone(), ParameterTarget::Absent);
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
            testee.executors.set(parameter_value.clone(), ParameterTarget::Present);


            let expected = None;
            let parameter_value = parameter::Executor {
                descriptor: ExecutorDescriptor {
                    results_url: expected.clone(),
                    ..parameter_value.descriptor
                }
            };

            testee.executors.set(parameter_value, ParameterTarget::Present);
            assert_eq!(testee.executors.len(), 1);
            assert_eq!(testee.executors[0].value.descriptor.results_url, expected);

            Ok(())
        }
    }

    mod set_all_present {
        use super::*;

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

            testee.ethernet_bridges.set(present_then_absent.clone(), ParameterTarget::Present);
            testee.ethernet_bridges.set(present_then_present.clone(), ParameterTarget::Present);
            testee.ethernet_bridges.set(absent_then_present.clone(), ParameterTarget::Absent);

            testee.ethernet_bridges.set_all_present([
                present_then_present.clone(),
                new_present.clone(),
                absent_then_present.clone()
            ]);

            assert_that!(testee.ethernet_bridges.values, unordered_elements_are![
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

    mod set_all_absent {
        use super::*;

        #[test]
        fn should_mark_all_parameters_as_absent() -> anyhow::Result<()> {

            fn parameter_value(id: &str) -> parameter::EthernetBridge {
                parameter::EthernetBridge { name: NetworkInterfaceName::try_from(id).unwrap() }
            }

            let initially_present = parameter_value("1");
            let initially_absent = parameter_value("2");

            let mut testee = PeerConfiguration::default();

            testee.ethernet_bridges.set(initially_present.clone(), ParameterTarget::Present);
            testee.ethernet_bridges.set(initially_absent.clone(), ParameterTarget::Absent);

            testee.ethernet_bridges.set_all_absent();

            assert_that!(testee.ethernet_bridges.values, unordered_elements_are![
                matches_pattern!(Parameter {
                    value: eq(&initially_present),
                    target: eq(&ParameterTarget::Absent),
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&initially_absent),
                    target: eq(&ParameterTarget::Absent),
                    ..
                }),
            ]);

            Ok(())
        }
    }
}
