use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use opendut_types::peer::PeerDescriptor;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::manager::ResourceManagerRef;
use crate::resource::storage::ResourcesStorageApi;

pub struct DetermineClusterPeersParams {
    pub resource_manager: ResourceManagerRef,
    pub cluster_id: ClusterId,
}

pub async fn determine_cluster_peers(params: DetermineClusterPeersParams) -> Result<Vec<PeerDescriptor>, DetermineClusterPeersError> {
    let DetermineClusterPeersParams { resource_manager, cluster_id } = params;

    let cluster_peers = resource_manager.resources(|resources| {
        let cluster_configuration = resources.get::<ClusterConfiguration>(cluster_id)
            .map_err(|source| DetermineClusterPeersError::Persistence { cluster_id, source })?
            .ok_or_else(|| DetermineClusterPeersError::ClusterNotFound(cluster_id))?;

        let peers = resources.list::<PeerDescriptor>()
            .map_err(|source| DetermineClusterPeersError::Persistence { cluster_id, source })?;

        let cluster_peers = peers.into_iter()
            .filter(|peer| peer.topology.devices.iter().any(|device| cluster_configuration.devices.contains(&device.id)))
            .collect();

        Ok::<_, DetermineClusterPeersError>(cluster_peers)
    }).await?;

    Ok(cluster_peers)
}

#[derive(thiserror::Error, Debug)]
pub enum DetermineClusterPeersError {
    #[error("Could not determine peers of cluster <{0}>, because the cluster was not found.")]
    ClusterNotFound(ClusterId),
    #[error("Error while accessing persistence for determining peers of cluster <{cluster_id}>.")]
    Persistence { cluster_id: ClusterId, #[source] source: PersistenceError },
}
