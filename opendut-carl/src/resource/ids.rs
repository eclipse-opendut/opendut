use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::state::PeerConnectionState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::resources::Id;

use crate::resource::resource::Resource;

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
impl IntoId<OldPeerConfiguration> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
}
impl IntoId<PeerConfiguration> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
}
impl IntoId<PeerConnectionState> for PeerId { 
    fn into_id(self) -> Id { Id::from(self.uuid) }
}