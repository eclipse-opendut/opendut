use opendut_model::cluster::{ClusterDescriptor, ClusterId, ClusterName};
use opendut_model::peer::PeerId;
use tracing::{debug, info};
use crate::resource::manager::Resources;
use crate::resource::manager::error::PersistenceError;
use crate::resource::manager::ResourcesStorageApi;

pub struct CreateClusterDescriptorParams {
    pub cluster_descriptor: ClusterDescriptor,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub fn create_cluster_descriptor(&mut self, params: CreateClusterDescriptorParams) -> Result<ClusterId, CreateClusterDescriptorError> {

        let cluster_id = params.cluster_descriptor.id;
        let cluster_name = Clone::clone(&params.cluster_descriptor.name);
        let leader_id = params.cluster_descriptor.leader;

        debug!("Creating cluster descriptor '{cluster_name}' <{cluster_id}>.");

        let leader_in_cluster = self.is_peer_in_cluster(leader_id, &params.cluster_descriptor.devices)
            .map_err(|source| CreateClusterDescriptorError::Persistence { cluster_id, cluster_name: cluster_name.clone(), source })?;

        if !leader_in_cluster {
            return Err(CreateClusterDescriptorError::LeaderNotInCluster { cluster_id, cluster_name, leader: leader_id });
        }

        self.insert(cluster_id, params.cluster_descriptor)
            .map_err(|source| CreateClusterDescriptorError::Persistence { cluster_id, cluster_name: cluster_name.clone(), source })?;

        info!("Successfully created cluster descriptor '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum CreateClusterDescriptorError {
    #[error("ClusterConfiguration '{cluster_name}' <{cluster_id}> could not be created, due to an error when accessing persistence")]
    Persistence {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        #[source] source: PersistenceError
    },
    #[error("ClusterConfiguration '{cluster_name}' <{cluster_id}> could not be created, because the specified leader peer <{leader}> is not part of the cluster")]
    LeaderNotInCluster {
        cluster_id: ClusterId,
        cluster_name: ClusterName,
        leader: PeerId,
    },
}

#[cfg(test)]
mod tests {
    use std::assert_matches::assert_matches;
    use std::collections::HashSet;
    use opendut_model::cluster::{ClusterDescriptor, ClusterId, ClusterName};
    use opendut_model::peer::PeerId;
    use crate::manager::testing::{ClusterFixture, PeerFixture};
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn should_reject_leader_not_in_cluster() -> anyhow::Result<()> {
        let (resource_manager, _cancel) = ResourceManager::new_in_memory();

        let peer_a = PeerFixture::new();
        let peer_b = PeerFixture::new();
        let unrelated_peer = PeerId::random();

        resource_manager.insert(peer_a.id, peer_a.descriptor.clone()).await?;
        resource_manager.insert(peer_b.id, peer_b.descriptor.clone()).await?;

        let cluster_id = ClusterId::random();
        let cluster_descriptor = ClusterDescriptor {
            id: cluster_id,
            name: ClusterName::try_from("TestCluster")?,
            leader: unrelated_peer,
            devices: HashSet::from([peer_a.device_1, peer_b.device_1]),
        };

        let result = resource_manager.resources_mut(async |resources| {
            resources.create_cluster_descriptor(CreateClusterDescriptorParams {
                cluster_descriptor,
            })
        }).await?;

        assert_matches!(result, Err(CreateClusterDescriptorError::LeaderNotInCluster { .. }));
        Ok(())
    }

    #[tokio::test]
    async fn should_accept_leader_that_is_in_cluster() -> anyhow::Result<()> {
        let (resource_manager, _cancel) = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;

        // ClusterFixture uses peer_a as leader and includes peer_a's devices — should succeed
        let new_cluster_id = ClusterId::random();
        let cluster_descriptor = ClusterDescriptor {
            id: new_cluster_id,
            name: ClusterName::try_from("ValidCluster")?,
            leader: cluster.peer_a.id,
            devices: HashSet::from([cluster.peer_a.device_1, cluster.peer_b.device_1]),
        };

        let result = resource_manager.resources_mut(async |resources| {
            resources.create_cluster_descriptor(CreateClusterDescriptorParams {
                cluster_descriptor,
            })
        }).await?;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), new_cluster_id);
        Ok(())
    }
}
