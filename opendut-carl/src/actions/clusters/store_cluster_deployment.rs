use crate::resources::manager::ResourcesManagerRef;
use opendut_carl_api::carl::cluster::StoreClusterDeploymentError;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName};
use tracing::error;

pub struct StoreClusterConfigurationParams {
    pub resources_manager: ResourcesManagerRef,
    pub deployment: ClusterDeployment,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn store_cluster_deployment(params: StoreClusterConfigurationParams) -> Result<ClusterId, StoreClusterDeploymentError> {

    async fn inner(params: StoreClusterConfigurationParams) -> Result<ClusterId, StoreClusterDeploymentError> {
        let StoreClusterConfigurationParams { resources_manager, deployment } = params;
        let cluster_id = deployment.id;

        resources_manager.resources_mut(|resources| {
            let cluster_name = resources.get::<ClusterConfiguration>(cluster_id)
                .map_err(|cause| StoreClusterDeploymentError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                .map(|cluster| cluster.name)
                .unwrap_or_else(|| ClusterName::try_from("unknown_cluster").unwrap());
            resources.insert(cluster_id, deployment)
                .map_err(|cause| StoreClusterDeploymentError::Internal { cluster_id, cluster_name: Some(cluster_name.clone()), cause: cause.to_string() })
        }).await?;

        Ok(cluster_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
