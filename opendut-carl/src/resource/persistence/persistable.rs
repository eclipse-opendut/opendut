use crate::resource::api::Resource;
use crate::resource::persistence::error::PersistenceResult;
use crate::resource::persistence::TableDefinition;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment};
use opendut_model::peer::configuration::{OldPeerConfiguration, PeerConfiguration, PeerConfigurationState};
use opendut_model::peer::state::PeerConnectionState;
use opendut_model::peer::PeerDescriptor;
use opendut_model::proto::ConversionError;
use prost::Message;
use std::fmt::Debug;
use opendut_model::test_suite::TestSuiteSourceDescriptor;

impl Persistable for ClusterDeployment {
    type Proto = opendut_model::proto::cluster::ClusterDeployment;
    const TABLE: &'static str = "cluster_deployment";
    const STORAGE: StorageKind = StorageKind::Persistent;
}

impl Persistable for ClusterDescriptor {
    type Proto = opendut_model::proto::cluster::ClusterDescriptor;
    const TABLE: &'static str = "cluster_descriptor";
    const STORAGE: StorageKind = StorageKind::Persistent;
}

impl Persistable for OldPeerConfiguration {
    type Proto = opendut_model::proto::peer::configuration::api::OldPeerConfiguration;
    const TABLE: &'static str = "old_peer_configuration";
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for PeerConfiguration {
    type Proto = opendut_model::proto::peer::configuration::api::PeerConfiguration;
    const TABLE: &'static str = "peer_configuration";
    /// Not persisted at the moment. A restart will cause a reconfiguration of all peers.
    /// The `assign_cluster()` method in the `ClusterManager` evaluates the current peer descriptors of the cluster and sends new peer configurations to the peers.
    /// It is called by the `ClusterManager` when a cluster deployment is created or when all peers of a cluster deployment are available.
    /// -> subscription triggers following chain: `schedule_redeploying_clusters_when_all_peers_become_available()` -> `rollout_all_clusters_containing_newly_available_peer()` -> `rollout_cluster_if_all_peers_available()`
    /// This is done to ensure that the cluster deployment is always up-to-date and that all peers are configured correctly.
    /// It is intended to be stored in a persistent storage once the peer configuration is stable to avoid migration issues.
    ///
    /// If you want to persist this, make sure the hashing algorithm in the implementations of `ParameterValue` is stable across Rust releases.
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for PeerConnectionState {
    type Proto = opendut_model::proto::peer::PeerConnectionState;
    const TABLE: &'static str = "peer_connection_state";
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for PeerDescriptor {
    type Proto = opendut_model::proto::peer::PeerDescriptor;
    const TABLE: &'static str = "peer_descriptor";
    const STORAGE: StorageKind = StorageKind::Persistent;
}

impl Persistable for PeerConfigurationState {
    type Proto = opendut_model::proto::peer::configuration::api::PeerConfigurationState;
    const TABLE: &'static str = "peer_configuration_state";
    const STORAGE: StorageKind = StorageKind::Volatile;
}

impl Persistable for TestSuiteSourceDescriptor {
    type Proto = opendut_model::proto::test_suite::TestSuiteSourceDescriptor;
    const TABLE: &'static str = "test_suite_source_descriptor";
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
