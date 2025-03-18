use crate::resource::manager::ResourceManagerRef;
use opendut_carl_api::carl::cluster::DeleteClusterConfigurationError;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use tracing::{debug, error, info};

pub struct DeleteClusterConfigurationParams {
    pub resource_manager: ResourceManagerRef,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_configuration(params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

    async fn inner(params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

        let cluster_id = params.cluster_id;
        let resource_manager = params.resource_manager;

        debug!("Deleting cluster configuration <{cluster_id}>.");

        let cluster_configuration = resource_manager.remove::<ClusterConfiguration>(cluster_id).await
            .map_err(|cause| DeleteClusterConfigurationError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
            .ok_or_else(|| DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id })?;

        let cluster_name = Clone::clone(&cluster_configuration.name);

        info!("Successfully deleted cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_configuration)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
