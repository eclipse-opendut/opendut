use crate::resource::api::Resource;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use opendut_model::peer::configuration::{OldPeerConfiguration, PeerConfiguration, EdgePeerConfigurationState};
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::resources::Id;

#[cfg(feature = "viper")]
use opendut_model::viper::{TestSuiteSourceDescriptor, TestSuiteSourceId};

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

impl ResourceId<EdgePeerConfigurationState> for PeerId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
    fn from_id(id: Id) -> Self {
        PeerId::from(id.value())
    }
}

#[cfg(feature = "viper")]
impl ResourceId<TestSuiteSourceDescriptor> for TestSuiteSourceId {
    fn into_id(self) -> Id {
        Id::from(self.uuid)
    }
    fn from_id(id: Id) -> Self {
        TestSuiteSourceId::from(id.value())
    }
}
