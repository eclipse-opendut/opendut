use opendut_model::cluster::ClusterDisplay;
use opendut_model::ShortName;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment, ClusterId, ClusterName};
use tracing::{debug, info};
use opendut_model::cluster::state::ClusterState;
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;

pub struct DeleteClusterDescriptorParams {
    pub cluster_id: ClusterId,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn delete_cluster_descriptor(&mut self, params: DeleteClusterDescriptorParams) -> Result<ClusterDescriptor, DeleteClusterDescriptorError> {

        let cluster_id = params.cluster_id;

        let cluster_deployment = self.get::<ClusterDeployment>(cluster_id)
            .map_err(|source| DeleteClusterDescriptorError::Persistence { cluster_id, cluster_name: None, source })?;

        match cluster_deployment {
            None => {
                debug!("Deleting cluster descriptor <{cluster_id}>.");

                let cluster_descriptor = self.remove::<ClusterDescriptor>(cluster_id)
                    .map_err(|source| DeleteClusterDescriptorError::Persistence { cluster_id, cluster_name: None, source })?
                    .ok_or_else(|| DeleteClusterDescriptorError::ClusterDescriptorNotFound { cluster_id })?;

                let cluster_name = Clone::clone(&cluster_descriptor.name);

                info!("Successfully deleted cluster descriptor '{cluster_name}' <{cluster_id}>.");

                Ok(cluster_descriptor)
            }
            Some(cluster_deployment) => {
                Err(DeleteClusterDescriptorError::ClusterDeploymentFound { cluster_id: cluster_deployment.id })
            }
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterDescriptorError {
    #[error("ClusterDescriptor <{cluster_id}> could not be deleted, because a ClusterDeployment with that ID still exists!")]
    ClusterDeploymentFound {
        cluster_id: ClusterId
    },
    #[error("ClusterDescriptor <{cluster_id}> could not be deleted, because a ClusterDescriptor with that ID does not exist!")]
    ClusterDescriptorNotFound {
        cluster_id: ClusterId
    },
    #[error(
        "ClusterDescriptor '{cluster_name}' <{cluster_id}> cannot be deleted when cluster is in state '{actual_state}'! A ClusterDescriptor can be deleted when cluster is in state: {required_states}",
        actual_state = actual_state.short_name(),
        required_states = ClusterState::short_names_joined(required_states),
    )]
    IllegalClusterState {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        actual_state: ClusterState,
        required_states: Vec<ClusterState>,
    },
    #[error("ClusterDescriptor {cluster} deleted with internal errors.", cluster=ClusterDisplay::new(cluster_name, cluster_id))]
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
    async fn block_deletion_of_cluster_descriptor_if_cluster_is_still_deployed() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        resource_manager.insert(cluster.id, ClusterDeployment { id: cluster.id }).await?;

        let result = resource_manager.resources_mut(async |resources| {
            resources.delete_cluster_descriptor(DeleteClusterDescriptorParams { cluster_id: cluster.id })
        }).await?;

        let Err(DeleteClusterDescriptorError::ClusterDeploymentFound { cluster_id }) = result
        else { panic!("Expected ClusterDeploymentFound error!") };

        assert_eq!(cluster_id, cluster.id);
        Ok(())
    }

    #[tokio::test]
    async fn delete_cluster_descriptor_when_cluster_is_not_deployed() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_cluster_descriptor(DeleteClusterDescriptorParams { cluster_id: cluster.id })
        ).await??;

        assert_eq!(result, cluster.configuration);
        Ok(())
    }
}
