use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::cluster::ClusterAssignment;
use crate::peer::executor::{ExecutorDescriptor, ExecutorKind};
use crate::util::net::NetworkInterfaceName;
use crate::OPENDUT_UUID_NAMESPACE;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OldPeerConfiguration {
    pub cluster_assignment: Option<ClusterAssignment>,
    pub network: PeerNetworkConfiguration,
    // Please add new fields into PeerConfiguration instead.
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerNetworkConfiguration {
    pub bridge_name: NetworkInterfaceName,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PeerConfiguration {
    pub executors: Vec<Parameter<ExecutorDescriptor>>,
    //TODO migrate more parameters
}
impl PeerConfiguration {
    pub fn insert<T: ParameterValue>(&mut self, value: T, target: ParameterTarget) {
        let parameter = Parameter {
            id: value.parameter_identifier(),
            dependencies: vec![], //TODO
            target,
            value,
        };

        T::insert_into_peer_configuration(parameter, self)
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

pub trait ParameterValue: Any + Hash + Sized {
    /// Unique identifier, which is ideally stable, too.
    /// A naive implementation for a `self` implementing `Hash` could look like this:
    /// ```
    /// # use std::hash::{DefaultHasher, Hash, Hasher};
    /// # use uuid::Uuid;
    /// # use opendut_types::peer::configuration::{Parameter, ParameterId, ParameterValue, PeerConfiguration};
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
    ///
    /// # fn insert_into_peer_configuration(parameter: Parameter<Self>, peer_configuration: &mut PeerConfiguration) { todo!() }
    /// # }
    /// ```
    /// However, ideally you use a stable subset of your data, which is still unique.
    fn parameter_identifier(&self) -> ParameterId;

    fn insert_into_peer_configuration(parameter: Parameter<Self>, peer_configuration: &mut PeerConfiguration);
}
impl ParameterValue for ExecutorDescriptor {
    fn parameter_identifier(&self) -> ParameterId {
        let mut hasher = DefaultHasher::new(); //ID not stable across Rust releases
        match &self.kind {
            ExecutorKind::Executable => self.kind.hash(&mut hasher),
            ExecutorKind::Container { name, .. } => name.hash(&mut hasher),
        }
        self.results_url.hash(&mut hasher);
        let id = hasher.finish();

        let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
        ParameterId(id)
    }

    fn insert_into_peer_configuration(parameter: Parameter<Self>, peer_configuration: &mut PeerConfiguration) {
        peer_configuration.executors.push(parameter);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::executor::ExecutorId;

    #[test]
    fn insert_value_in_peer_configuration() {
        let mut peer_configuration = PeerConfiguration::default();

        let value = ExecutorDescriptor {
            id: ExecutorId::random(),
            kind: ExecutorKind::Executable,
            results_url: None
        };
        let target = ParameterTarget::Present;
        peer_configuration.insert(value.clone(), target);

        assert_eq!(peer_configuration.executors.len(), 1);

        let executor_target = peer_configuration.executors.first().unwrap();
        assert_eq!(executor_target.value, value);
        assert_eq!(executor_target.target, target);
    }
}
