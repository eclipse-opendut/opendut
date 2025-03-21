use crate::settings::vpn::Vpn;
use opendut_carl_api::carl::cluster::DeleteClusterDeploymentError;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
use crate::resource::api::resources::Resources;
use crate::resource::storage::ResourcesStorageApi;

pub struct DeleteClusterDeploymentParams {
    pub cluster_id: ClusterId,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn delete_cluster_deployment(&mut self, params: DeleteClusterDeploymentParams) -> Result<ClusterDeployment, DeleteClusterDeploymentError> {

        let DeleteClusterDeploymentParams { cluster_id } = params;

        let (deployment, cluster) =
            self.remove::<ClusterDeployment>(cluster_id)
                .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                .map(|deployment| {
                    let configuration = self.get::<ClusterConfiguration>(cluster_id)
                        .map_err(|cause| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?;
                    Ok((deployment, configuration))
                })
                .ok_or(DeleteClusterDeploymentError::ClusterDeploymentNotFound { cluster_id })??;

        if let Some(cluster) = cluster {
            if let Vpn::Enabled { vpn_client } = self.global.get::<Vpn>() {
                vpn_client.delete_cluster(cluster_id).await
                    .map_err(|error| DeleteClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster.name.clone()), cause: error.to_string() })?;
            }

            // TODO: unassign cluster for each peer
        }

        Ok(deployment)
    }
}
