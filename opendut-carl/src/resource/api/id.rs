use crate::resource::api::Resource;
use opendut_types::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::state::PeerConnectionState;
use opendut_types::peer::{PeerDescriptor, PeerId};
use opendut_types::resources::Id;


pub trait ResourceId<R: Resource> {
    fn into_id(self) -> Id;
    fn from_id(id: Id) -> Self;
}

impl ResourceId<ClusterDeployment> for ClusterId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }
    fn from_id(id: Id) -> Self {
        ClusterId::from(id.value())
    }
}
impl ResourceId<ClusterDescriptor> for ClusterId {
    fn into_id(self) -> Id {
        Id::from(self.0)
    }

    fn from_id(id: Id) -> Self {
        ClusterId::from(id.value())
    }
}
impl ResourceId<PeerDescriptor> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
    fn from_id(id: Id) -> Self {
        PeerId::from(id.value())
    }
}
impl ResourceId<OldPeerConfiguration> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
    fn from_id(id: Id) -> Self {
        PeerId::from(id.value())
    }
}
impl ResourceId<PeerConfiguration> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
    fn from_id(id: Id) -> Self {
        PeerId::from(id.value())
    }
}
impl ResourceId<PeerConnectionState> for PeerId {
    fn into_id(self) -> Id { Id::from(self.uuid) }
    fn from_id(id: Id) -> Self {
        PeerId::from(id.value())
    }
}
