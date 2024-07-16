use std::any::Any;
use std::fmt::Debug;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2};
use opendut_types::peer::PeerDescriptor;
use opendut_types::peer::state::PeerState;
use opendut_types::topology::DeviceDescriptor;
use crate::persistence::model::cluster_configuration::PersistableClusterConfiguration;
use crate::persistence::model::cluster_deployment::PersistableClusterDeployment;
use crate::persistence::model::device_descriptor::PersistableDeviceDescriptor;
use crate::persistence::model::peer_configuration2::PersistablePeerConfiguration2;
use crate::persistence::model::peer_configuration::PersistablePeerConfiguration;
use crate::persistence::model::peer_descriptor::PersistablePeerDescriptor;
use crate::persistence::model::peer_state::PersistablePeerState;
use crate::persistence::model::PersistableConversionError;

pub trait Resource: Any + Send + Sync
+ Debug
+ TryFrom<
    Self::Persistable,
    Error=PersistableConversionError<Self::Persistable, Self>
> {
    type Persistable: crate::persistence::model::Persistable<Self>;
}

impl Resource for ClusterConfiguration {
    type Persistable = PersistableClusterConfiguration;
}
impl Resource for ClusterDeployment {
    type Persistable = PersistableClusterDeployment;
}
impl Resource for DeviceDescriptor {
    type Persistable = PersistableDeviceDescriptor;
}
impl Resource for PeerConfiguration {
    type Persistable = PersistablePeerConfiguration;
}
impl Resource for PeerConfiguration2 {
    type Persistable = PersistablePeerConfiguration2;
}
impl Resource for PeerDescriptor {
    type Persistable = PersistablePeerDescriptor;
}
impl Resource for PeerState {
    type Persistable = PersistablePeerState;
}
