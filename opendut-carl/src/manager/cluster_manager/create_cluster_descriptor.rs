use opendut_model::cluster::{ClusterDeployment, ClusterDescriptor, ClusterId, ClusterName};
use tracing::{debug, info};
use opendut_model::peer::{PeerDescriptor, PeerId};
use crate::manager::cluster_manager::{DeleteClusterDescriptorError, DeleteClusterDescriptorParams};
use crate::manager::testing::ClusterFixture;
use crate::resource::api::resources::Resources;
use crate::resource::manager::ResourceManager;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;

pub struct CreateClusterDescriptorParams {
    pub cluster_descriptor: ClusterDescriptor,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn create_cluster_descriptor(&mut self, params: CreateClusterDescriptorParams) -> Result<ClusterId, CreateClusterDescriptorError> {
        let CreateClusterDescriptorParams { cluster_descriptor } = params;

        let cluster_id = cluster_descriptor.id;
        let cluster_name = Clone::clone(&cluster_descriptor.name);

        debug!("Creating cluster descriptor '{cluster_name}' <{cluster_id}>.");

        let peer_descriptor = self.get::<PeerDescriptor>(cluster_descriptor.leader)
            .map_err(|source| CreateClusterDescriptorError::LeaderNotInCluster { cluster_id, cluster_name: cluster_name.clone(), leader_id: cluster_descriptor.leader })?;

        self.insert(cluster_id, cluster_descriptor)
            .map_err(|source| CreateClusterDescriptorError::Persistence { cluster_id, cluster_name: cluster_name.clone(), source })?;

        info!("Successfully created cluster descriptor '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateClusterDescriptorError {
    #[error("ClusterConfigration '{cluster_name}' <{cluster_id}> could not be created, because the specified leader peer <{leader_id}> is not part of the cluster")]
    LeaderNotInCluster {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        leader_id: PeerId,
    },
    #[error("ClusterConfigration '{cluster_name}' <{cluster_id}> could not be created, due to an error when accessing persistence")]
    Persistence {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        #[source] source: PersistenceError
    },
}


#[cfg(test)]
mod tests {
    use crate::manager::testing::PeerFixture;
    use super::*;

    #[tokio::test]
    async fn should_reject_when_leader_is_not_in_cluster() -> anyhow::Result<()> {
        let resource_manager = ResourceManager::new_in_memory();
        let peer1 = PeerFixture::new();
        resource_manager.insert(peer1.id, peer1.descriptor).await?;
        let peer2 = PeerFixture::new();
        resource_manager.insert(peer2.id, peer2.descriptor).await?;

        todo!()
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

        assert_eq!(result, cluster.descriptor);
        Ok(())
    }
}
