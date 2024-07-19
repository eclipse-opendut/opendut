use std::any::Any;
use std::fmt::Debug;

use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2};
use opendut_types::peer::PeerDescriptor;
use opendut_types::peer::state::PeerState;
use opendut_types::topology::DeviceDescriptor;

use crate::persistence::model::Persistable;

pub trait Resource: Any + Send + Sync + Sized + Debug + Persistable {}

impl Resource for ClusterConfiguration {}
impl Resource for ClusterDeployment {}
impl Resource for DeviceDescriptor {}
impl Resource for PeerConfiguration {}
impl Resource for PeerConfiguration2 {}
impl Resource for PeerDescriptor {}
impl Resource for PeerState {}
