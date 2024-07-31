use tracing::{debug, error, info};
pub use opendut_carl_api::carl::cluster::{
    CreateClusterConfigurationError,
    DeleteClusterConfigurationError
};
use opendut_types::cluster::{ClusterConfiguration, ClusterId};

use crate::resources::manager::ResourcesManagerRef;

pub struct CreateClusterConfigurationParams {
    pub resources_manager: ResourcesManagerRef,
    pub cluster_configuration: ClusterConfiguration,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn create_cluster_configuration(params: CreateClusterConfigurationParams) -> Result<ClusterId, CreateClusterConfigurationError> {

    async fn inner(params: CreateClusterConfigurationParams) -> Result<ClusterId, CreateClusterConfigurationError> {

        let cluster_id = params.cluster_configuration.id;
        let cluster_name = Clone::clone(&params.cluster_configuration.name);
        let resources_manager = params.resources_manager;

        debug!("Creating cluster configuration '{cluster_name}' <{cluster_id}>.");

        resources_manager.resources_mut(|resources| {
            resources.insert(cluster_id, params.cluster_configuration)
                .map_err(|cause| CreateClusterConfigurationError::Internal { cluster_id, cluster_name: cluster_name.clone(), cause: cause.to_string() })
        }).await?;

        info!("Successfully created cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

pub struct DeleteClusterConfigurationParams {
    pub resources_manager: ResourcesManagerRef,
    pub cluster_id: ClusterId,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_cluster_configuration(params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

    async fn inner(params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

        let cluster_id = params.cluster_id;
        let resources_manager = params.resources_manager;

        debug!("Deleting cluster configuration <{cluster_id}>.");

        let cluster_configuration = resources_manager.resources_mut(|resources| {
            resources.remove::<ClusterConfiguration>(cluster_id)
                .map_err(|cause| DeleteClusterConfigurationError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                .ok_or_else(|| DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id })
        }).await?;

        let cluster_name = Clone::clone(&cluster_configuration.name);

        info!("Successfully deleted cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_configuration)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
