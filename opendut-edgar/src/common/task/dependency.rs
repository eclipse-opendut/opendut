use opendut_model::peer::configuration::{ParameterId, ParameterVariant, PeerConfiguration};
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use tracing::debug;

pub struct PeerConfigurationDependencyResolver {
    /// Parameters with dependencies that need to be completed.
    open: HashMap<ParameterId, ParameterVariantWithDependencies>,
    /// Parameters that have been executed successfully, initially empty.
    completed: HashMap<ParameterId, ParameterVariantWithDependencies>,
    /// Parameters that have NOT been executed successfully, initially empty.
    failed: HashMap<ParameterId, ParameterVariantWithDependencies>,
    current: Option<(ParameterId, ParameterVariantWithDependencies)>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ParameterVariantWithDependencies {
    pub id: ParameterId,
    pub parameter: ParameterVariant,
    pub dependencies: HashSet<ParameterId>,
}

impl ParameterVariantWithDependencies {
    pub fn unresolved_dependencies(&self, completed: &HashSet<ParameterId>) -> HashSet<ParameterId> {
        self.dependencies
            .difference(completed)
            .cloned()
            .collect()
    }
}

impl Hash for ParameterVariantWithDependencies {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match &self.parameter {
            ParameterVariant::DeviceInterface(parameter) => parameter.id.hash(state),
            ParameterVariant::EthernetBridge(parameter) => parameter.id.hash(state),
            ParameterVariant::Executor(parameter) => parameter.id.hash(state),
            ParameterVariant::GreInterface(parameter) => parameter.id.hash(state),
            ParameterVariant::JoinedInterface(parameter) => parameter.id.hash(state),
            ParameterVariant::RemotePeerConnectionCheck(parameter) => parameter.id.hash(state),
            ParameterVariant::CanConnections(parameter) => parameter.id.hash(state),
            ParameterVariant::CanBridges(parameter) => parameter.id.hash(state),
            ParameterVariant::CanLocalRoutes(parameter) => parameter.id.hash(state),
        }
    }
}


impl PeerConfigurationDependencyResolver {
    pub fn new(peer_configuration: PeerConfiguration) -> Self {
        let all_parameters = peer_configuration.all_parameters()
            .into_iter()
            .map(|(id, parameter)| {
                let dependencies = parameter.dependencies();
                (id, ParameterVariantWithDependencies { id, parameter, dependencies })
            }).collect::<HashMap<_, _>>();

        Self { open: all_parameters, completed: Default::default(), failed: Default::default(), current: None }
    }

    /*
        Using a variation of 
        - depth-first traversal in post-order (see https://en.wikipedia.org/wiki/Tree_traversal#Depth-first_search)
        - or Kahn's algorithm: https://en.wikipedia.org/wiki/Topological_sorting#Kahn's_algorithm


     */
    pub fn next_parameter(&mut self) -> Option<ParameterVariant>{
        // assume the last parameter was completed successful and not marked as failed
        self.mark_current_parameter_as_succeeded();

        let next = self.determine_next_parameter();

        if let Some(next_param) = next.clone() {
            self.open.remove(&next_param.id);
            self.current = Some((next_param.id, next_param))
        }

        next.map(|parameter| { parameter.parameter })        
    }

    fn completed_ids(&self) -> HashSet<ParameterId> {
        self.completed.keys().cloned().collect()
    }

    fn determine_next_parameter(&mut self) -> Option<ParameterVariantWithDependencies> {
        let candidates = self.open.values().filter_map(|parameter| {
            if parameter.dependencies.is_empty() {
                Some(parameter.clone())
            } else {
                let unresolved_dependencies = parameter.unresolved_dependencies(&self.completed_ids());
                if unresolved_dependencies.is_empty() {
                    Some(parameter.clone())
                } else {
                    None
                }
            }
        }).collect::<HashSet<_>>();

        candidates
            .into_iter()
            .next()
    }

    fn mark_current_parameter_as_succeeded(&mut self) {
        if let Some(current) = self.current.take() {
            self.completed.insert(current.0, current.1);
        }
        // if there is no current parameter, it could have been marked as failed previously
    }

    pub fn mark_current_parameter_as_failed(&mut self) {
        if let Some(current) = self.current.take() {
            self.failed.insert(current.0, current.1);
        }
    }

    pub fn success(&mut self) -> bool {
        let outcome = self.open.is_empty() && self.failed.is_empty() && self.current.is_none();
        debug!("Dependency resolver completed. Success: {}. Completed parameters: {}. Failed parameters: {}. Unfulfilled parameters: {}. Current parameter: {:?}",
            outcome,
            self.completed.len(),
            self.failed.len(),
            self.open.len(),
            self.current,
        );
        outcome
    }
    
    pub fn unfulfilled(&self) -> Vec<ParameterVariantWithDependencies> {
        self.open.values().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use opendut_model::peer::configuration::{parameter, ParameterTarget};
    use opendut_model::util::net::{NetworkInterfaceConfiguration, NetworkInterfaceDescriptor, NetworkInterfaceId, NetworkInterfaceName};

    impl PeerConfigurationDependencyResolver {
        pub fn done(&mut self) -> bool {
            let cannot_choose_another_parameter = self.determine_next_parameter().is_none();
            cannot_choose_another_parameter && self.current.is_none()
        }
    }

    struct PeerConfigurationDependencyResolverFixture {
        resolver: PeerConfigurationDependencyResolver,
        bridge_name: NetworkInterfaceName,
        bridge_old_name: NetworkInterfaceName,
        config: PeerConfiguration,
    }
    impl PeerConfigurationDependencyResolverFixture {
        fn new() -> Self {
            let bridge_old_name = NetworkInterfaceName::try_from("br-old").unwrap();
            let bridge_name = NetworkInterfaceName::try_from("br-opendut").unwrap();
            let parameter_bridge_old = parameter::EthernetBridge { name: bridge_old_name.clone() };
            let parameter_bridge_new = parameter::EthernetBridge { name: bridge_name.clone() };

            let dut_name = NetworkInterfaceName::try_from("dut0").unwrap();
            let dut_descriptor = NetworkInterfaceDescriptor {
                id: NetworkInterfaceId::random(),
                name: dut_name.clone(),
                configuration: NetworkInterfaceConfiguration::Ethernet,
            };
            let parameter_eth_device = parameter::DeviceInterface { descriptor: dut_descriptor };
            let parameter_join = parameter::InterfaceJoinConfig { name: dut_name, bridge: bridge_name.clone() };

            let mut config = PeerConfiguration::default();
            // add old bridge as present and check if set_all_present adds a dependency to remove the old bridge before adding the new
            config.ethernet_bridges.set(parameter_bridge_old.clone(), ParameterTarget::Present, vec![]);
            let mut joined_interfaces_dependencies = config.ethernet_bridges.set_all_present(vec![parameter_bridge_new.clone()], vec![]);

            let device_dependency = config.device_interfaces.set(parameter_eth_device, ParameterTarget::Present, vec![]);

            joined_interfaces_dependencies.push(device_dependency);
            config.joined_interfaces.set(parameter_join.clone(), ParameterTarget::Present, joined_interfaces_dependencies.clone());

            let resolver = PeerConfigurationDependencyResolver::new(config.clone());

            PeerConfigurationDependencyResolverFixture {
                resolver,
                bridge_name,
                bridge_old_name,
                config,
            }
        }
    }

    #[test]
    fn determine_task_order_happy_flow() {
        fn find_bridge_parameter_task_position(tasks: &[ParameterVariant], bridge_name: NetworkInterfaceName) -> Option<usize> {
            tasks.iter().enumerate().find_map(|(pos, param)| {
                if let ParameterVariant::EthernetBridge(bridge) = param {
                    if bridge.value.name == bridge_name {
                        return Some(pos)
                    }
                };
                None
            })
        }

        let mut testee = PeerConfigurationDependencyResolverFixture::new();


        let mut tasks: Vec<ParameterVariant> = vec![];
        while let Some(next_parameter) = testee.resolver.next_parameter() {
            tasks.push(next_parameter);            
        }
        assert_eq!(tasks.len(), 4);
        assert!(testee.resolver.done());
        assert!(testee.resolver.success());
        let position_remove_old_bridge = find_bridge_parameter_task_position(&tasks, testee.bridge_old_name)
            .expect("Expected bridge old parameter to be found in task list.");
        let position_new_bridge = find_bridge_parameter_task_position(&tasks, testee.bridge_name)
            .expect("Expected bridge new parameter to be found in task list.");
        assert!(position_remove_old_bridge < position_new_bridge, "The task of removing the old bridge must precede the addition of a new bridge.");
    }

    #[test_log::test]
    fn determine_task_order_when_one_task_fails() {
        let mut testee = PeerConfigurationDependencyResolverFixture::new();
        let mut tasks: Vec<ParameterVariant> = vec![];
        while let Some(next_parameter) = testee.resolver.next_parameter() {
            if matches!(next_parameter, ParameterVariant::DeviceInterface { .. }) {
                testee.resolver.mark_current_parameter_as_failed();
            }
            tasks.push(next_parameter);
        }
        assert_eq!(tasks.len(), 3);
        assert!(testee.resolver.done());
        assert!(!testee.resolver.success());
        let config = testee.config.joined_interfaces.clone();
        let id = config.values().next().unwrap().id.clone();

        assert!(
            testee.resolver.open.contains_key(&id),
            "The only unfulfilled parameter should be the joined interface that depends on the failed device interface."
        );

    }
}
