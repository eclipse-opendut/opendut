use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2};
use opendut_types::peer::state::PeerState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::resources::Id;

use crate::resources::resource::Resource;

pub trait IntoId<R: Resource> {
    fn into_id(self) -> Id;
}


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
impl IntoId<PeerDescriptor> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
}
impl IntoId<PeerState> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
}
impl IntoId<PeerConfiguration> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
}
impl IntoId<PeerConfiguration2> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
}
