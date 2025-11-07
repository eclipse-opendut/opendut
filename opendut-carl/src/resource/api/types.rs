use crate::resource::api::id::ResourceId;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use opendut_model::peer::configuration::{OldPeerConfiguration, PeerConfiguration, EdgePeerConfigurationState};
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::{PeerDescriptor, PeerId};
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

#[cfg(feature = "viper")]
use opendut_model::viper::{TestSuiteRunDeployment, TestSuiteRunDescriptor, TestSuiteRunId, TestSuiteSourceDescriptor, TestSuiteSourceId};

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

impl Resource for EdgePeerConfigurationState {
    type Id = PeerId;
}

#[cfg(feature = "viper")]
impl Resource for TestSuiteSourceDescriptor {
    type Id = TestSuiteSourceId;
}

#[cfg(feature = "viper")]
impl Resource for TestSuiteRunDescriptor {
    type Id = TestSuiteRunId;
}

#[cfg(feature = "viper")]
impl Resource for TestSuiteRunDeployment {
    type Id = TestSuiteRunId;
}
