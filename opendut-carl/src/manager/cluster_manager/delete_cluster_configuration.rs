use opendut_types::cluster::ClusterDisplay;
use opendut_types::ShortName;
use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment, ClusterId, ClusterName};
use tracing::{debug, info};
use opendut_types::cluster::state::ClusterState;
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;

pub struct DeleteClusterConfigurationParams {
    pub cluster_id: ClusterId,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn delete_cluster_configuration(&mut self, params: DeleteClusterConfigurationParams) -> Result<ClusterConfiguration, DeleteClusterConfigurationError> {

        let cluster_id = params.cluster_id;

        let cluster_deployment = self.get::<ClusterDeployment>(cluster_id)
            .map_err(|source| DeleteClusterConfigurationError::Persistence { cluster_id, cluster_name: None, source })?;

        match cluster_deployment {
            None => {
                debug!("Deleting cluster configuration <{cluster_id}>.");

                let cluster_configuration = self.remove::<ClusterConfiguration>(cluster_id)
                    .map_err(|source| DeleteClusterConfigurationError::Persistence { cluster_id, cluster_name: None, source })?
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
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterConfigurationError {
    #[error("ClusterConfiguration <{cluster_id}> could not be deleted, because a ClusterDeployment with that ID still exists!")]
    ClusterDeploymentFound {
        cluster_id: ClusterId
    },
    #[error("ClusterConfiguration <{cluster_id}> could not be deleted, because a ClusterConfiguration with that ID does not exist!")]
    ClusterConfigurationNotFound {
        cluster_id: ClusterId
    },
    #[error(
        "ClusterConfiguration '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A ClusterConfiguration can be deleted when cluster is in state: {required_states}",
        actual_state = actual_state.short_name(),
        required_states = ClusterState::short_names_joined(required_states),
    )]
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    #[error("ClusterConfiguration {cluster} deleted with internal errors.", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
    Persistence {
        cluster_id: ClusterId,
        cluster_name: Option<ClusterName>,
        #[source] source: PersistenceError,
    }
}

#[cfg(test)]
mod tests {
    use crate::manager::testing::ClusterFixture;
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn block_deletion_of_cluster_configuration_if_cluster_is_still_deployed() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        resource_manager.insert(cluster.id, ClusterDeployment { id: cluster.id }).await?;

        let result = resource_manager.resources_mut(async |resources| {
            resources.delete_cluster_configuration(DeleteClusterConfigurationParams { cluster_id: cluster.id })
        }).await?;

        let Err(DeleteClusterConfigurationError::ClusterDeploymentFound { cluster_id }) = result
        else { panic!("Expected ClusterDeploymentFound error!") };

        assert_eq!(cluster_id, cluster.id);
        Ok(())
    }

    #[tokio::test]
    async fn delete_cluster_configuration_when_cluster_is_not_deployed() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_cluster_configuration(DeleteClusterConfigurationParams { cluster_id: cluster.id })
        ).await??;

        assert_eq!(result, cluster.configuration);
        Ok(())
    }
}
