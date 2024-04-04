use serde::{Deserialize, Serialize};
use crate::cluster::ClusterAssignment;
use crate::peer::executor::ExecutorDescriptors;
use crate::util::net::{NetworkInterfaceName};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerConfiguration {
    pub executors: ExecutorDescriptors,
    pub cluster_assignment: Option<ClusterAssignment>,
    pub network: PeerNetworkConfiguration,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct PeerNetworkConfiguration {
    pub bridge_name: NetworkInterfaceName,
}