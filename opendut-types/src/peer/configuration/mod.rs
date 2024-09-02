use serde::{Deserialize, Serialize};

use crate::cluster::ClusterAssignment;
use crate::peer::executor::ExecutorDescriptor;
use crate::util::net::NetworkInterfaceName;

mod parameter;
pub use parameter::*;
use crate::peer::ethernet::EthernetBridge;

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
    pub ethernet_bridges: Vec<Parameter<EthernetBridge>>,
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

        T::peer_configuration_field(self)
            .push(parameter);
    }
}
