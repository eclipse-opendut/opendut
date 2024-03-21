use crate::cluster::ClusterAssignment;
use crate::peer::executor::ExecutorDescriptors;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PeerConfiguration {
    pub executors: ExecutorDescriptors,
    pub cluster_assignment: Option<ClusterAssignment>,
}
