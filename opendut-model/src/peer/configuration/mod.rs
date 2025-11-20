use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::ops::{Deref, DerefMut, Not};

pub mod api;
pub use crate::peer::configuration::api::*;

pub mod parameter;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct PeerConfiguration {
    pub device_interfaces: ParameterField<parameter::DeviceInterface>,
    pub ethernet_bridges: ParameterField<parameter::EthernetBridge>,
    pub executors: ParameterField<parameter::Executor>,
    pub gre_interfaces: ParameterField<parameter::GreInterfaceConfig>,
    pub joined_interfaces: ParameterField<parameter::InterfaceJoinConfig>,
    pub remote_peer_connection_checks: ParameterField<parameter::RemotePeerConnectionCheck>,
    pub can_connections: ParameterField<parameter::CanConnection>,
    pub can_bridges: ParameterField<parameter::CanBridge>,
    pub can_local_routes: ParameterField<parameter::CanLocalRoute>,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub enum ParameterVariant {
    DeviceInterface(Box<Parameter<parameter::DeviceInterface>>),
    EthernetBridge(Box<Parameter<parameter::EthernetBridge>>),
    Executor(Box<Parameter<parameter::Executor>>),
    GreInterface(Box<Parameter<parameter::GreInterfaceConfig>>),
    JoinedInterface(Box<Parameter<parameter::InterfaceJoinConfig>>),
    RemotePeerConnectionCheck(Box<Parameter<parameter::RemotePeerConnectionCheck>>),
    CanConnections(Box<Parameter<parameter::CanConnection>>),
    CanBridges(Box<Parameter<parameter::CanBridge>>),
    CanLocalRoutes(Box<Parameter<parameter::CanLocalRoute>>),
}

impl ParameterVariant {
    pub fn dependencies(&self) -> HashSet<ParameterId> {
        match self {
            ParameterVariant::DeviceInterface(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::EthernetBridge(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::Executor(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::GreInterface(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::JoinedInterface(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::RemotePeerConnectionCheck(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::CanConnections(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::CanBridges(parameter) => { parameter.dependencies.clone() }
            ParameterVariant::CanLocalRoutes(parameter) => { parameter.dependencies.clone() }
        }
    }
    pub fn target(&self) -> ParameterTarget {
        match self {
            ParameterVariant::DeviceInterface(parameter) => parameter.target,
            ParameterVariant::EthernetBridge(parameter) => parameter.target,
            ParameterVariant::Executor(parameter) => parameter.target,
            ParameterVariant::GreInterface(parameter) => parameter.target,
            ParameterVariant::JoinedInterface(parameter) => parameter.target,
            ParameterVariant::RemotePeerConnectionCheck(parameter) => parameter.target,
            ParameterVariant::CanConnections(parameter) => parameter.target,
            ParameterVariant::CanBridges(parameter) => parameter.target,
            ParameterVariant::CanLocalRoutes(parameter) => parameter.target,
        }
    }
    pub fn id(&self) -> ParameterId {
        match self {
            ParameterVariant::DeviceInterface(parameter) => parameter.id,
            ParameterVariant::EthernetBridge(parameter) => parameter.id,
            ParameterVariant::Executor(parameter) => parameter.id,
            ParameterVariant::GreInterface(parameter) => parameter.id,
            ParameterVariant::JoinedInterface(parameter) => parameter.id,
            ParameterVariant::RemotePeerConnectionCheck(parameter) => parameter.id,
            ParameterVariant::CanConnections(parameter) => parameter.id,
            ParameterVariant::CanBridges(parameter) => parameter.id,
            ParameterVariant::CanLocalRoutes(parameter) => parameter.id,
        }
    }
}

impl PeerConfiguration {
    pub fn all_parameters(&self) -> HashMap<ParameterId, ParameterVariant> {
        let PeerConfiguration {
            device_interfaces,
            ethernet_bridges,
            executors,
            gre_interfaces,
            joined_interfaces,
            remote_peer_connection_checks,
            can_connections,
            can_bridges,
            can_local_routes,            
        } = self.clone();

        device_interfaces.values.into_iter().map(|(id, parameter) | { (id, ParameterVariant::DeviceInterface(Box::new(parameter))) })
            .chain(ethernet_bridges.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::EthernetBridge(Box::new(parameter))) }))
            .chain(executors.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::Executor(Box::new(parameter))) }))
            .chain(gre_interfaces.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::GreInterface(Box::new(parameter))) }))
            .chain(joined_interfaces.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::JoinedInterface(Box::new(parameter))) }))
            .chain(remote_peer_connection_checks.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::RemotePeerConnectionCheck(Box::new(parameter))) }))
            .chain(can_connections.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::CanConnections(Box::new(parameter))) }))
            .chain(can_bridges.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::CanBridges(Box::new(parameter))) }))
            .chain(can_local_routes.values.into_iter().map(|(id, parameter)| { (id, ParameterVariant::CanLocalRoutes(Box::new(parameter))) }))
            .collect()
    }

    pub fn remove_parameters(
        &mut self,
        obsolete_parameter_ids: &HashSet<ParameterId>,
    ) {
        self.device_interfaces.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.ethernet_bridges.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.executors.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.gre_interfaces.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.joined_interfaces.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.remote_peer_connection_checks.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.can_connections.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.can_bridges.retain(|id, _| !obsolete_parameter_ids.contains(id));
        self.can_local_routes.retain(|id, _| !obsolete_parameter_ids.contains(id));
    }
}

impl<V: ParameterValue> ParameterField<V> {
    /// Set all parameters of a type to be present.
    /// Parameters that were previously in the PeerConfiguration,
    /// but aren't in the new list, will be set to absent.
    pub fn set_all_present(&mut self, values: impl IntoIterator<Item=V>, dependencies: HashSet<ParameterId>) -> HashSet<ParameterId> {

        let new_present_parameters = values.into_iter()
            .map(|value| Self::create_parameter(value, ParameterTarget::Present, dependencies.clone()))
            .collect::<HashSet<_>>();
        let new_present_ids = new_present_parameters.iter()
            .map(|p| p.id)
            .collect::<HashSet<_>>();

        let parameters_to_set_absent = self.iter_mut()
            .map(|(_id, parameter_ref)| parameter_ref.to_owned())
            .filter(|obsolete_parameter| new_present_ids.contains(&obsolete_parameter.id).not())
            .collect::<HashSet<_>>();

        let mut dependency_on_absents= HashSet::<ParameterId>::new();
        for parameter in parameters_to_set_absent {
            let absent_parameter = Parameter {
                target: ParameterTarget::Absent,
                ..parameter.clone()
            };
            let id = self.set_parameter(absent_parameter);
            dependency_on_absents.insert(id);
        }

        let mut parameter_ids: HashSet<ParameterId> = HashSet::new();
        for mut parameter in new_present_parameters {
            let current_dependencies = parameter.dependencies;
            parameter.dependencies = current_dependencies.union(&dependency_on_absents).cloned().collect::<HashSet<_>>();
            let id = self.set_parameter(parameter);
            parameter_ids.insert(id);
        }
        parameter_ids
    }

    /// Set all parameters of a type to be absent.
    pub fn set_all_absent(&mut self) -> Vec<ParameterId> {
        let parameters_to_set_absent = self.clone();
        let mut parameter_ids: Vec<ParameterId> = vec![];

        for parameter in parameters_to_set_absent {
            let absent_parameter = Parameter {
                target: ParameterTarget::Absent,
                ..parameter
            };
            let id = self.set_parameter(absent_parameter);
            parameter_ids.push(id);
        }
        parameter_ids
    }

    /// Set an individual parameter to be present/absent
    pub fn set(&mut self, value: V, target: ParameterTarget, dependencies: HashSet<ParameterId>) -> ParameterId {
        let parameter = Self::create_parameter(value, target, dependencies);
        self.set_parameter(parameter)
    }


    fn set_parameter(&mut self, parameter: Parameter<V>) -> ParameterId {
        let id = parameter.id;
        self.values.insert(parameter.id, parameter);
        id
    }

    fn create_parameter(value: V, target: ParameterTarget, dependencies: HashSet<ParameterId>) -> Parameter<V> {
        Parameter {
            id: value.parameter_identifier(),
            dependencies,
            target,
            value,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ParameterField<V: ParameterValue> {
    pub values: HashMap<ParameterId, Parameter<V>>,
}
impl<V: ParameterValue> Default for ParameterField<V> {
    fn default() -> Self {
        Self { values: HashMap::new() }
    }
}
impl<V: ParameterValue> Deref for ParameterField<V> {
    type Target = HashMap<ParameterId, Parameter<V>>;

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
        self.values.into_values()
            .collect::<Vec<_>>()
            .into_iter()
    }
}
impl<V: ParameterValue> FromIterator<Parameter<V>> for ParameterField<V> {
    fn from_iter<T: IntoIterator<Item=Parameter<V>>>(iter: T) -> Self {
        Self {
            values: iter.into_iter()
                .map(|parameter| (parameter.id, parameter))
                .collect(),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind, ResultsUrl};
    use crate::util::net::NetworkInterfaceName;
    use googletest::prelude::*;

    mod set {
        use super::*;

        #[test]
        fn should_replace_a_previous_parameter_when_it_is_set_another_time() -> anyhow::Result<()> {

            let parameter_value = parameter::EthernetBridge { name: NetworkInterfaceName::try_from("br-opendut")? };

            let mut testee = PeerConfiguration::default();
            testee.ethernet_bridges.set(parameter_value.clone(), ParameterTarget::Present, HashSet::new());


            testee.ethernet_bridges.set(parameter_value.clone(), ParameterTarget::Present, HashSet::new());
            assert_eq!(testee.ethernet_bridges.len(), 1);

            testee.ethernet_bridges.set(parameter_value.clone(), ParameterTarget::Absent, HashSet::new());
            assert_eq!(testee.ethernet_bridges.len(), 1);
            let id = parameter_value.parameter_identifier();
            let first_ethernet_bridge = testee.ethernet_bridges.get(&id).unwrap();
            assert_eq!(first_ethernet_bridge.target, ParameterTarget::Absent);

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
            testee.executors.set(parameter_value.clone(), ParameterTarget::Present, HashSet::new());


            let expected = None;
            let parameter_value = parameter::Executor {
                descriptor: ExecutorDescriptor {
                    results_url: expected.clone(),
                    ..parameter_value.descriptor
                }
            };
            let id = parameter_value.parameter_identifier();

            testee.executors.set(parameter_value, ParameterTarget::Present, HashSet::new());
            assert_eq!(testee.executors.len(), 1);
            assert_eq!(testee.executors.get(&id).unwrap().value.descriptor.results_url, expected);

            Ok(())
        }
    }

    mod set_all_present {
        use super::*;

        #[test]
        fn should_mark_obsolete_parameters_as_absent_and_retain_or_set_other_parameters_as_present() -> anyhow::Result<()> {
            // ARRANGE
            fn parameter_value(id: &str) -> parameter::EthernetBridge {
                parameter::EthernetBridge { name: NetworkInterfaceName::try_from(id).unwrap() }
            }

            let present_then_absent = parameter_value("1");
            let present_then_present = parameter_value("2");
            let new_present = parameter_value("3");
            let absent_then_present = parameter_value("4");

            let mut testee = PeerConfiguration::default();

            testee.ethernet_bridges.set(present_then_absent.clone(), ParameterTarget::Present, HashSet::new());
            testee.ethernet_bridges.set(present_then_present.clone(), ParameterTarget::Present, HashSet::new());
            testee.ethernet_bridges.set(absent_then_present.clone(), ParameterTarget::Absent, HashSet::new());

            // ACT
            testee.ethernet_bridges.set_all_present([
                    present_then_present.clone(),
                    new_present.clone(),
                    absent_then_present.clone()
                ],
                HashSet::new()
            );

            // ASSERT
            assert_that!(testee.ethernet_bridges.values.into_values().collect::<Vec<_>>(), unordered_elements_are![
                matches_pattern!(Parameter {
                    value: eq(&present_then_absent),
                    target: eq(&ParameterTarget::Absent),
                    dependencies: eq(&HashSet::new()),  // removing something should not have dependencies
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&present_then_present),
                    target: eq(&ParameterTarget::Present),
                    dependencies: eq(&HashSet::from_iter(vec![present_then_absent.parameter_identifier()])),
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&new_present),
                    target: eq(&ParameterTarget::Present),
                    dependencies: eq(&HashSet::from_iter(vec![present_then_absent.parameter_identifier()])),
                    ..
                }),
                matches_pattern!(Parameter {
                    value: eq(&absent_then_present),
                    target: eq(&ParameterTarget::Present),
                    dependencies: eq(&HashSet::from_iter(vec![present_then_absent.parameter_identifier()])),
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

            testee.ethernet_bridges.set(initially_present.clone(), ParameterTarget::Present, HashSet::new());
            testee.ethernet_bridges.set(initially_absent.clone(), ParameterTarget::Absent, HashSet::new());

            testee.ethernet_bridges.set_all_absent();

            assert_that!(testee.ethernet_bridges.values.into_values().collect::<Vec<_>>(), unordered_elements_are![
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
