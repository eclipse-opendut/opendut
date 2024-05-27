use std::any::Any;

use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::configuration::{PeerConfiguration, PeerConfiguration2};
use opendut_types::peer::PeerDescriptor;
use opendut_types::peer::state::PeerState;
use opendut_types::topology::DeviceDescriptor;

pub trait Resource: Any + Send + Sync {}

impl Resource for ClusterConfiguration {}
impl Resource for ClusterDeployment {}
impl Resource for PeerDescriptor {}
impl Resource for PeerConfiguration {}
impl Resource for PeerConfiguration2 {}
impl Resource for PeerState {}
impl Resource for DeviceDescriptor {}
