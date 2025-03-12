use crate::resources::manager::ResourcesManagerRef;
use opendut_carl_api::carl::cluster::CreateClusterConfigurationError;
use opendut_types::cluster::{ClusterConfiguration, ClusterId};
use tracing::{debug, error, info};
use crate::resources::storage::ResourcesStorageApi;

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
        }).await
        .map_err(|cause| CreateClusterConfigurationError::Internal { cluster_id, cluster_name: cluster_name.clone(), cause: cause.to_string() })??;

        info!("Successfully created cluster configuration '{cluster_name}' <{cluster_id}>.");

        Ok(cluster_id)
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}


#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use opendut_types::cluster::ClusterName;
    use crate::actions::testing::PeerFixture;
    use super::*;

    /// Our data model does not match well with RDBMS, since we send full models to the backend, whereas RDBMSs work with diffs.
    /// As such, this verifies that we correctly diff the full models to delete removed child elements.
    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn updating_should_correctly_remove_devices_from_the_database() -> anyhow::Result<()> {
        let peer_a = PeerFixture::new();
        let peer_b = PeerFixture::new();

        let db = crate::persistence::database::testing::spawn_and_connect_resources_manager().await?;
        let resources_manager = db.resources_manager;

        resources_manager.insert(peer_a.id, peer_a.descriptor).await?;
        resources_manager.insert(peer_b.id, peer_b.descriptor).await?;

        let cluster_id = ClusterId::random();
        let cluster_configuration_a = ClusterConfiguration {
            id: cluster_id,
            name: ClusterName::try_from("Cluster1")?,
            leader: peer_a.id,
            devices: HashSet::from([peer_a.device_1, peer_a.device_2, peer_b.device_1]),
        };
        resources_manager.insert(cluster_id, cluster_configuration_a.clone()).await?;

        assert_eq!(
            resources_manager.get::<ClusterConfiguration>(cluster_id).await?.unwrap(),
            cluster_configuration_a
        );

        let cluster_configuration_b = ClusterConfiguration {
            devices: HashSet::from([peer_a.device_1]),
            ..cluster_configuration_a
        };
        resources_manager.insert(cluster_id, cluster_configuration_b.clone()).await?;

        assert_eq!(
            resources_manager.get::<ClusterConfiguration>(cluster_id).await?.unwrap(),
            cluster_configuration_b
        );

        Ok(())
    }
}
