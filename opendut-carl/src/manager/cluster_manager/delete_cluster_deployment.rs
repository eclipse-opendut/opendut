use crate::resource::manager::ResourceManagerRef;
use crate::settings::vpn::Vpn;
use opendut_carl_api::carl::cluster::DeleteClusterDeploymentError;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use tracing::error;
use crate::resource::storage::ResourcesStorageApi;

pub struct DeleteClusterDeploymentParams {
    pub resource_manager: ResourceManagerRef,
    pub vpn: Vpn,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_deployment(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

    async fn inner(params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {
        let DeleteClusterDeploymentParams { resource_manager, vpn, cluster_id } = params;

        let (deployment, cluster) = resource_manager
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

            // TODO: unassign cluster for each peer
        }

        Ok(deployment)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
