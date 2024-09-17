use crate::cluster::ClusterAssignment;
use crate::peer::ethernet::EthernetBridge;
use crate::peer::executor::ExecutorDescriptor;

mod parameter;
pub use parameter::*;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OldPeerConfiguration {
    pub cluster_assignment: Option<ClusterAssignment>,
    // Please add new fields into PeerConfiguration instead.
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
            .push(parameter); //TODO don't insert, if already contained? Or maybe set the previous element absent?
    }
}
