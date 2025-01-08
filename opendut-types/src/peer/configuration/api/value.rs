use crate::peer::configuration::{parameter, Parameter, ParameterId};
use crate::peer::configuration::PeerConfiguration;
use crate::OPENDUT_UUID_NAMESPACE;
use std::any::Any;
use std::hash::{DefaultHasher, Hash, Hasher};
use uuid::Uuid;

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
    /// # fn peer_configuration_field(peer_configuration: &mut PeerConfiguration) -> &mut Vec<Parameter<Self>> { todo!() }
    /// # }
    /// ```
    /// However, ideally you use a stable subset of your data, which is still unique.
    fn parameter_identifier(&self) -> ParameterId;

    fn peer_configuration_field(peer_configuration: &mut PeerConfiguration) -> &mut Vec<Parameter<Self>>;
}

impl ParameterValue for parameter::DeviceInterface {
    fn parameter_identifier(&self) -> ParameterId {
        ParameterId(self.descriptor.id.uuid)
    }
    fn peer_configuration_field(peer_configuration: &mut PeerConfiguration) -> &mut Vec<Parameter<Self>>  {
        &mut peer_configuration.device_interfaces
    }
}
impl ParameterValue for parameter::EthernetBridge {
    fn parameter_identifier(&self) -> ParameterId {
        let mut hasher = DefaultHasher::new(); //ID not stable across Rust releases
        self.name.name().hash(&mut hasher);
        let id = hasher.finish();

        let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
        ParameterId(id)
    }
    fn peer_configuration_field(peer_configuration: &mut PeerConfiguration) -> &mut Vec<Parameter<Self>> {
        &mut peer_configuration.ethernet_bridges
    }
}
impl ParameterValue for parameter::Executor {
    fn parameter_identifier(&self) -> ParameterId {
        ParameterId(self.descriptor.id.uuid)
    }
    fn peer_configuration_field(peer_configuration: &mut PeerConfiguration) -> &mut Vec<Parameter<Self>>  {
        &mut peer_configuration.executors
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::peer::configuration::ParameterTarget;
    use crate::peer::executor::{ExecutorDescriptor, ExecutorId, ExecutorKind};

    #[test]
    fn insert_value_in_peer_configuration() {
        let mut peer_configuration = PeerConfiguration::default();

        let value = parameter::Executor {
            descriptor: ExecutorDescriptor {
                id: ExecutorId::random(),
                kind: ExecutorKind::Executable,
                results_url: None
            }
        };
        let target = ParameterTarget::Present;
        peer_configuration.set(value.clone(), target);

        assert_eq!(peer_configuration.executors.len(), 1);

        let executor_target = peer_configuration.executors.first().unwrap();
        assert_eq!(executor_target.value, value);
        assert_eq!(executor_target.target, target);
    }
}
