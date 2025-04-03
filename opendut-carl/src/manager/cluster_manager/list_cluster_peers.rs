use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use opendut_types::peer::PeerDescriptor;


impl Resources<'_> {
    pub async fn list_cluster_peers(&self, cluster_id: ClusterId) -> Result<Vec<PeerDescriptor>, ListClusterPeersError> {
        let cluster_configuration = self.get::<ClusterConfiguration>(cluster_id)
            .map_err(|source| ListClusterPeersError::Persistence { cluster_id, source })?
            .ok_or_else(|| ListClusterPeersError::ClusterNotFound(cluster_id))?;

        let peers = self.list::<PeerDescriptor>()
            .map_err(|source| ListClusterPeersError::Persistence { cluster_id, source })?;

        let cluster_peers = peers.into_values()
            .filter(|peer| peer.topology.devices.iter().any(|device| cluster_configuration.devices.contains(&device.id)))
            .collect();

        Ok(cluster_peers)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListClusterPeersError {
    #[error("Could not determine peers of cluster <{0}>, because the cluster was not found.")]
    ClusterNotFound(ClusterId),
    #[error("Error while accessing persistence for determining peers of cluster <{cluster_id}>.")]
    Persistence { cluster_id: ClusterId, #[source] source: PersistenceError },
}
