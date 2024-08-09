use std::any::Any;
use std::fmt::Debug;

use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2};
use opendut_types::peer::state::PeerState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::topology::{DeviceDescriptor, DeviceId};

use crate::resources::ids::IntoId;

pub trait Resource: Any + Send + Sync + Debug + Clone {
    type Id: IntoId<Self>;
}

impl Resource for ClusterConfiguration {
    type Id = ClusterId;
}
impl Resource for ClusterDeployment {
    type Id = ClusterId;
}
impl Resource for DeviceDescriptor {
    type Id = DeviceId;
}
impl Resource for PeerConfiguration {
    type Id = PeerId;
}
impl Resource for PeerConfiguration2 {
    type Id = PeerId;
}
impl Resource for PeerDescriptor {
    type Id = PeerId;
}
impl Resource for PeerState {
    type Id = PeerId;
}
