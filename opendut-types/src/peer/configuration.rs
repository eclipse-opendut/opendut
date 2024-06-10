use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cluster::ClusterAssignment;
use crate::topology::AccessoryDescriptor;
use crate::OPENDUT_UUID_NAMESPACE;
use crate::peer::executor::ExecutorDescriptor;
use crate::util::net::NetworkInterfaceName;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerConfiguration {
    pub cluster_assignment: Option<ClusterAssignment>,
    pub network: PeerNetworkConfiguration,
    // Please add new fields into PeerConfiguration2 instead.
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerNetworkConfiguration {
    pub bridge_name: NetworkInterfaceName,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeerConfiguration2 {
    pub executors: Vec<Parameter<ExecutorDescriptor>>,
    pub accessories: Vec<Parameter<AccessoryDescriptor>>,
    //TODO migrate more parameters
}
impl PeerConfiguration2 {
    pub fn insert_executor(&mut self, value: ExecutorDescriptor, target: ParameterTarget) { //TODO more generic solution
        let parameter = Parameter {
            id: value.parameter_identifier(),
            dependencies: vec![], //TODO
            target,
            value,
        };

        self.executors.push(parameter);
    }

    pub fn insert_accessory(&mut self, value: AccessoryDescriptor, target: ParameterTarget) { //TODO more generic solution
        let parameter = Parameter {
            id: value.parameter_identifier(),
            dependencies: vec![], //TODO
            target,
            value,
        };

        self.accessories.push(parameter);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parameter<V: ParameterValue> {
    pub id: ParameterId,
    pub dependencies: Vec<ParameterId>,
    pub target: ParameterTarget,
    pub value: V,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ParameterId(pub Uuid);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterTarget {
    Present,
    Absent,
}

pub trait ParameterValue: Any + Hash {
    /// Unique identifier, which is ideally stable, too.
    /// A naive implementation for a `self` implementing `Hash` could look like this:
    /// ```
    /// # use std::hash::{DefaultHasher, Hash, Hasher};
    /// # use uuid::Uuid;
    /// # use opendut_types::peer::configuration::{ParameterId, ParameterValue};
    /// # use opendut_types::OPENDUT_UUID_NAMESPACE;
    ///
    /// # #[derive(Hash)]
    /// # struct Something;
    ///
    /// # impl ParameterValue for Something {
    /// fn parameter_identifier(&self) -> ParameterId {
    ///     let mut hasher = DefaultHasher::new();
    ///     self.hash(&mut hasher);
    ///     let id = hasher.finish();
    ///
    ///     let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
    ///     ParameterId(id)
    /// }
    /// # }
    /// ```
    /// However, ideally you use a stable subset of your data, which is still unique.
    fn parameter_identifier(&self) -> ParameterId;
}
impl ParameterValue for ExecutorDescriptor {
    fn parameter_identifier(&self) -> ParameterId {
        let mut hasher = DefaultHasher::new(); //ID not stable across Rust releases
        match self {
            ExecutorDescriptor::Executable => self.hash(&mut hasher),
            ExecutorDescriptor::Container { name, .. } => name.hash(&mut hasher),
        }
        let id = hasher.finish();

        let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
        ParameterId(id)
    }
}

impl ParameterValue for AccessoryDescriptor {
    fn parameter_identifier(&self) -> ParameterId {
        let mut hasher = DefaultHasher::new(); //ID not stable across Rust releases
        self.name.hash(&mut hasher);
        let id = hasher.finish();

        let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
        ParameterId(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_value_in_peer_configuration2() {
        let mut peer_configuration = PeerConfiguration2::default();

        let value = ExecutorDescriptor::Executable;
        let target = ParameterTarget::Present;
        peer_configuration.insert_executor(value.clone(), target);

        assert_eq!(peer_configuration.executors.len(), 1);

        let executor_target = peer_configuration.executors.first().unwrap();
        assert_eq!(executor_target.value, value);
        assert_eq!(executor_target.target, target);
    }
}
