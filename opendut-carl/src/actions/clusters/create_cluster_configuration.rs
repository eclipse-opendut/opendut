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
    use crate::actions::clusters::testing::ClusterFixture;
    use super::*;

    /// Our data model does not match well with RDBMS, since we send full models to the backend, whereas RDBMSs work with diffs.
    /// As such, this verifies that we correctly diff the full models to delete removed child elements.
    #[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
    #[tokio::test]
    async fn updating_should_correctly_remove_devices_from_the_database() -> anyhow::Result<()> {
        let db = crate::persistence::database::testing::spawn_and_connect_resources_manager().await?;
        let resources_manager = db.resources_manager;
        let cluster = ClusterFixture::create(resources_manager.clone()).await?;
        resources_manager.insert(cluster.id, cluster.configuration.clone()).await?;

        assert_eq!(
            resources_manager.get::<ClusterConfiguration>(cluster.id).await?.unwrap(),
            cluster.configuration
        );

        let updated_cluster_configuration = ClusterConfiguration {
            devices: HashSet::from([cluster.peer_a.device_1]),
            ..cluster.configuration
        };
        resources_manager.insert(cluster.id, updated_cluster_configuration.clone()).await?;

        assert_eq!(
            resources_manager.get::<ClusterConfiguration>(cluster.id).await?.unwrap(),
            updated_cluster_configuration
        );

        Ok(())
    }
}
