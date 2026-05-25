use crate::resource::api::id::ResourceId;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId};
use opendut_model::peer::configuration::{PeerConfiguration, EdgePeerConfigurationState};
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::secret::{SecretDescriptor, SecretId};
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;

#[cfg(feature = "viper")]
use opendut_model::viper::{ViperRunDeployment, ViperTestRunDescriptor, ViperTestId, ViperSourceDescriptor, ViperSourceId, ViperRunId};

pub trait Resource: Any + Send + Sync + Debug + Clone {
    type Id: ResourceId<Self> + Clone + Hash + PartialEq + Eq + Debug;
}

impl Resource for ClusterDeployment {
    type Id = ClusterId;
}
impl Resource for ClusterDescriptor {
    type Id = ClusterId;
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
impl Resource for ViperSourceDescriptor {
    type Id = ViperSourceId;
}

#[cfg(feature = "viper")]
impl Resource for ViperTestRunDescriptor {
    type Id = ViperTestId;
}

#[cfg(feature = "viper")]
impl Resource for ViperRunDeployment {
    type Id = ViperRunId;
}

impl Resource for SecretDescriptor {
    type Id = SecretId;
}
