use crate::resource::manager::ResourceManagerRef;
use opendut_carl_api::carl::cluster::DeleteClusterConfigurationError;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId};
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

        let cluster_deployment = resource_manager.get::<ClusterDeployment>(cluster_id)
            .await
            .map_err(|cause| DeleteClusterConfigurationError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?;

        match cluster_deployment {
            None => {
                debug!("Deleting cluster configuration <{cluster_id}>.");

                let cluster_configuration = resource_manager.remove::<ClusterConfiguration>(cluster_id).await
                    .map_err(|cause| DeleteClusterConfigurationError::Internal { cluster_id, cluster_name: None, cause: cause.to_string() })?
                    .ok_or_else(|| DeleteClusterConfigurationError::ClusterConfigurationNotFound { cluster_id })?;

                let cluster_name = Clone::clone(&cluster_configuration.name);

                info!("Successfully deleted cluster configuration '{cluster_name}' <{cluster_id}>.");

                Ok(cluster_configuration)
            }
            Some(cluster_deployment) => {
                Err(DeleteClusterConfigurationError::ClusterDeploymentFound { cluster_id: cluster_deployment.id })
            }
        }

    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}

#[cfg(test)]
mod tests {
    use crate::actions::clusters::testing::ClusterFixture;
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn block_deletion_of_cluster_configuration_if_cluster_is_still_deployed() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        resource_manager.insert(cluster.id, ClusterDeployment { id: cluster.id }).await?;

        let result = delete_cluster_configuration(DeleteClusterConfigurationParams { resource_manager, cluster_id: cluster.id }).await;

        let expected_error = Err(DeleteClusterConfigurationError::ClusterDeploymentFound { cluster_id: cluster.id });
        assert_eq!(expected_error, result);
        Ok(())
    }

    #[tokio::test]
    async fn delete_cluster_configuration_when_cluster_is_not_deployed() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        let result = delete_cluster_configuration(DeleteClusterConfigurationParams { resource_manager, cluster_id: cluster.id }).await;

        let expected_result = Ok(cluster.configuration);
        assert_eq!(expected_result, result);

        Ok(())
    }
}