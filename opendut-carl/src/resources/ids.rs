use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::peer::configuration::PeerConfiguration;
use opendut_types::peer::state::PeerState;
use opendut_types::topology::{DeviceDescriptor, DeviceId};

use crate::resources::{Id, IntoId};

impl IntoId<ClusterConfiguration> for ClusterId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}
impl IntoId<ClusterDeployment> for ClusterId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}

impl IntoId<DeviceDescriptor> for DeviceId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}

impl IntoId<PeerDescriptor> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}
impl IntoId<PeerState> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}
impl IntoId<PeerConfiguration> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}