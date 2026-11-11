use std::collections::HashSet;

use crate::resource::manager::Resources;
use crate::resource::manager::error::PersistenceError;
use crate::resource::manager::ResourcesStorageApi;
use opendut_model::cluster::{ClusterDescriptor, ClusterId};
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::topology::DeviceId;


impl Resources<'_> {
    pub fn list_cluster_peers(&self, cluster_id: ClusterId) -> Result<Vec<PeerDescriptor>, ListClusterPeersError> {
        let cluster_descriptor = self.get::<ClusterDescriptor>(cluster_id)
            .map_err(|source| ListClusterPeersError::Persistence { cluster_id, source })?
            .ok_or_else(|| ListClusterPeersError::ClusterNotFound(cluster_id))?;

        let peers = self.list::<PeerDescriptor>()
            .map_err(|source| ListClusterPeersError::Persistence { cluster_id, source })?;

        let cluster_peers = peers.into_values()
            .filter(|peer| peer.topology.devices.iter().any(|device| cluster_descriptor.devices.contains(&device.id)))
            .collect();

        Ok(cluster_peers)
    }

    pub fn is_peer_in_cluster(&self, peer_id: PeerId, cluster_devices: &HashSet<DeviceId>) -> Result<bool, PersistenceError> {
        let peer = self.get::<PeerDescriptor>(peer_id)?;

        let is_in_cluster = peer
            .map(|peer| {
                peer.topology.devices.iter()
                    .any(|device| cluster_devices.contains(&device.id))
            })
            .unwrap_or(false);

        Ok(is_in_cluster)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListClusterPeersError {
    #[error("Could not determine peers of cluster <{0}>, because the cluster was not found.")]
    ClusterNotFound(ClusterId),
    #[error("Error while accessing persistence for determining peers of cluster <{cluster_id}>.")]
    Persistence { cluster_id: ClusterId, #[source] source: PersistenceError },
}
