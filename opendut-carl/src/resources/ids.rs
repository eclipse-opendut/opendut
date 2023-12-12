use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::topology::{Device, DeviceId};

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

impl IntoId<Device> for DeviceId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}

impl IntoId<PeerDescriptor> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
}
