use crate::actions;
use crate::actions::UnassignClusterParams;
use crate::resources::manager::ResourcesManagerRef;
use crate::vpn::Vpn;
use opendut_carl_api::carl::cluster::DeleteClusterDeploymentError;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use opendut_types::peer::{PeerDescriptor, PeerId};
use std::ops::Not;
use std::sync::Arc;
use tracing::error;
use crate::resources::storage::ResourcesStorageApi;

pub struct DeleteClusterDeploymentParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_deployment(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

    async fn inner(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {
        let DeleteClusterDeploymentParams { resources_manager, vpn, cluster_id } = params;

        let (deployment, cluster) = resources_manager
            .resources_mut(|resources| {
                resources.remove::<ClusterDeployment>(cluster_id)
                    .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                    .map(|deployment| {
                        let configuration = resources.get::<ClusterConfiguration>(cluster_id)
                            .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?;
                        Ok((deployment, configuration))
                    }).transpose()
            }).await
            .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })??
            .ok_or(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })?;

        if let Some(cluster) = cluster {
            if let Vpn::Enabled { vpn_client } = vpn {
                vpn_client.delete_cluster(cluster_id).await
                    .map_err(|error| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: error.to_string() })?;
            }

            {
                let all_peers = resources_manager.list::<PeerDescriptor>().await
                    .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: cause.to_string() })?;

                let member_ids = all_peers.into_iter()
                    .filter(|peer|
                        peer.topology.devices.iter()
                            .filter(|device| cluster.devices.contains(&device.id))
                            .collect::<Vec<_>>()
                            .is_empty().not()
                    )
                    .map(|peer| peer.id)
                    .collect::<Vec<PeerId>>();

                for member_id in member_ids {
                    actions::unassign_cluster(UnassignClusterParams {
                        resources_manager: Arc::clone(&resources_manager),
                        peer_id: member_id,
                    }).await
                        .map_err(|cause| {
                            let message = format!("Failure while unassigning cluster <{cluster_id}> from peer <{member_id}>.");
                            error!("{}\n  {cause}", message);
                            DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: message }
                        })?;
                }
            }
        }

        Ok(deployment)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
