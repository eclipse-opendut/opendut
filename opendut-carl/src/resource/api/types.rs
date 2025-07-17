use crate::resource::api::id::ResourceId;
use opendut_types::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration, PeerConfigurationState};
use opendut_types::peer::state::PeerConnectionState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Resource: Any + Send + Sync + Debug + Clone {
    type Id: ResourceId<Self> + Clone + Hash + PartialEq + Eq + Debug;
}

impl Resource for ClusterDeployment {
    type Id = ClusterId;
}
impl Resource for ClusterDescriptor {
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

impl Resource for PeerConfigurationState {
    type Id = PeerId;
}
