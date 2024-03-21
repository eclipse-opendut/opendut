use crate::peer::executor::ExecutorDescriptors;

#[derive(Clone, Debug, PartialEq)]
pub struct PeerConfiguration {
    pub executors: ExecutorDescriptors,
}
