use crate::resource::api::Resource;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::TableDefinition;
use opendut_types::cluster::{ClusterDescriptor, ClusterDeployment};
use opendut_types::peer::configuration::{OldPeerConfiguration, PeerConfiguration};
use opendut_types::peer::state::PeerConnectionState;
use opendut_types::peer::PeerDescriptor;
use opendut_types::proto::ConversionError;
use prost::Message;
use std::fmt::Debug;


impl Persistable for ClusterDeployment {
    type Proto = opendut_types::proto::cluster::ClusterDeployment;
    const TABLE: &'static str = "cluster_deployment";
    const STORAGE: StorageKind = StorageKind::Persistent;
}

impl Persistable for ClusterDescriptor {
    type Proto = opendut_types::proto::cluster::ClusterDescriptor;
    const TABLE: &'static str = "cluster_descriptor";
    const STORAGE: StorageKind = StorageKind::Persistent;
}

impl Persistable for OldPeerConfiguration {
    type Proto = opendut_types::proto::peer::configuration::api::OldPeerConfiguration;
    const TABLE: &'static str = "old_peer_configuration";
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for PeerConfiguration {
    type Proto = opendut_types::proto::peer::configuration::api::PeerConfiguration;
    const TABLE: &'static str = "peer_configuration";
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for PeerConnectionState {
    type Proto = opendut_types::proto::peer::PeerConnectionState;
    const TABLE: &'static str = "peer_connection_state";
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for PeerDescriptor {
    type Proto = opendut_types::proto::peer::PeerDescriptor;
    const TABLE: &'static str = "peer_descriptor";
    const STORAGE: StorageKind = StorageKind::Persistent;
}


pub trait Persistable: Send + Sync + Sized + Debug + Resource {
    type Proto: Message + Default + From<Self> + TryInto<Self, Error=ConversionError>;
    const TABLE: &'static str;
    const STORAGE: StorageKind;


    const TABLE_DEFINITION: TableDefinition<'_> = TableDefinition::new(Self::TABLE);

    fn try_from_bytes(bytes: Vec<u8>) -> PersistenceResult<Self> {
        let value = Self::Proto::decode(bytes.as_slice())?;
        let value: Self = value.try_into()?;
        Ok(value)
    }
}

pub enum StorageKind { Persistent, Volatile }
