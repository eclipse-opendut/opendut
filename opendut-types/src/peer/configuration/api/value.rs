use crate::peer::configuration::{parameter, ParameterId};
use crate::OPENDUT_UUID_NAMESPACE;
use std::any::Any;
use std::fmt::Display;
use std::hash::{DefaultHasher, Hash, Hasher};
use uuid::Uuid;

pub trait ParameterValue: Any + Clone + PartialEq + Eq + Hash + Sized {
    /// Unique identifier, which is ideally stable, too.
    /// A naive implementation for a `self` implementing `Hash` could look like this:
    /// ```
    /// # use std::hash::{DefaultHasher, Hash, Hasher};
    /// # use uuid::Uuid;
    /// # use opendut_types::peer::configuration::{Parameter, ParameterField, ParameterId, ParameterValue, PeerConfiguration};
    /// # use opendut_types::OPENDUT_UUID_NAMESPACE;
    ///
    /// # #[derive(Clone, PartialEq, Eq, Hash)]
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

impl ParameterValue for parameter::DeviceInterface {
    fn parameter_identifier(&self) -> ParameterId {
        ParameterId(self.descriptor.id.uuid)
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
}
impl ParameterValue for parameter::Executor {
    fn parameter_identifier(&self) -> ParameterId {
        ParameterId(self.descriptor.id.uuid)
    }
}

fn parameter_value_hash<T: ParameterValue + Display>(value: &T) -> ParameterId {
    let repr = value.to_string();
    let mut hasher = DefaultHasher::new(); //ID not stable across Rust releases
    repr.hash(&mut hasher);
    let id = hasher.finish();
    let id = Uuid::new_v5(&OPENDUT_UUID_NAMESPACE, &id.to_le_bytes());
    ParameterId(id)
}

impl ParameterValue for parameter::GreInterfaceConfig {
    fn parameter_identifier(&self) -> ParameterId {
        parameter_value_hash(self)
    }
}

impl ParameterValue for parameter::InterfaceJoinConfig {
    fn parameter_identifier(&self) -> ParameterId {
        parameter_value_hash(self)
    }
}

impl ParameterValue for parameter::RemotePeerConnectionCheck {
    fn parameter_identifier(&self) -> ParameterId {
        parameter_value_hash(self)
    }
}


#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use std::str::FromStr;
    use crate::peer::configuration::parameter::GreInterfaceConfig;
    use super::*;
    use crate::peer::configuration::{ParameterTarget, PeerConfiguration};
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
        peer_configuration.executors.set(value.clone(), target, vec![]);

        assert_eq!(peer_configuration.executors.len(), 1);

        let all_executor_parameters = peer_configuration.executors.into_iter().collect::<Vec<_>>();
        let executor_parameter = all_executor_parameters.first().unwrap();
        assert_eq!(executor_parameter.value, value);
        assert_eq!(executor_parameter.target, target);
    }
    
    #[test]
    fn test_parameter_hash_value() -> anyhow::Result<()> {
        let foo = GreInterfaceConfig {
            local_ip: Ipv4Addr::from_str("192.168.0.1")?,
            remote_ip: Ipv4Addr::from_str("192.168.0.2")?,
        };
        let hash1 = parameter_value_hash(&foo);
        let hash2 = parameter_value_hash(&foo);
        assert_eq!(hash1, hash2);
        
        Ok(())
    }
}
