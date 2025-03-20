use crate::resource::api::id::ResourceId;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::state::PeerConnectionState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Resource: Any + Send + Sync + Debug + Clone {
    type Id: ResourceId<Self> + Clone + Hash + PartialEq + Eq + Debug;
}

impl Resource for ClusterConfiguration {
    type Id = ClusterId;
}
impl Resource for ClusterDeployment {
    type Id = ClusterId;
}
impl Resource for OldPeerConfiguration {
    type Id = PeerId;
}
impl Resource for PeerConfiguration {
    type Id = PeerId;
}
impl Resource for PeerDescriptor {
    type Id = PeerId;
}
impl Resource for PeerConnectionState {
    type Id = PeerId;
}
