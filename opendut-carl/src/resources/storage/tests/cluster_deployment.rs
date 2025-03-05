use opendut_types::cluster::{ClusterConfiguration, ClusterDeployment};
use opendut_types::peer::PeerDescriptor;
use crate::persistence::database;
use crate::resources::manager::{ResourcesManager, ResourcesManagerRef};

#[tokio::test]
async fn should_persist_cluster_deployment_in_memory() -> anyhow::Result<()> {
    let resources_manager = ResourcesManager::new_in_memory();
    should_persist_cluster_deployment(resources_manager).await
}

#[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
#[tokio::test]
async fn should_persist_cluster_deployment_in_database() -> anyhow::Result<()> {
    let db = database::testing::spawn_and_connect_resources_manager().await?;
    should_persist_cluster_deployment(db.resources_manager).await
}

async fn should_persist_cluster_deployment(resources_manager: ResourcesManagerRef) -> anyhow::Result<()> {
    // Arrange
    let peer_descriptor = super::peer_descriptor::peer_descriptor()?;
    resources_manager.insert::<PeerDescriptor>(peer_descriptor.id, peer_descriptor.clone()).await?;
    let peer_descriptor_unused = super::peer_descriptor::peer_descriptor()?;
    resources_manager.insert::<PeerDescriptor>(peer_descriptor_unused.id, peer_descriptor_unused.clone()).await?;

    let cluster_configuration = super::cluster_configuration::cluster_configuration(
        peer_descriptor.id,
        peer_descriptor.topology.devices.into_iter().map(|device| device.id).collect()
    )?;
    resources_manager.insert::<ClusterConfiguration>(cluster_configuration.id, cluster_configuration.clone()).await?;

    let testee = ClusterDeployment {
        id: cluster_configuration.id,
    };

    let result = resources_manager.get::<ClusterDeployment>(testee.id).await?;
    assert!(result.is_none());
    let result = resources_manager.list::<ClusterDeployment>().await?;
    assert!(result.is_empty());

    // Act
    resources_manager.insert::<ClusterDeployment>(testee.id, testee.clone()).await?;

    // Assert
    let result = resources_manager.get::<ClusterDeployment>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));
    let result = resources_manager.list::<ClusterDeployment>().await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result.first(), Some(&testee));

    let result = resources_manager.remove::<ClusterDeployment>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));

    let result = resources_manager.get::<ClusterDeployment>(testee.id).await?;
    assert!(result.is_none());
    let result = resources_manager.list::<ClusterDeployment>().await?;
    assert!(result.is_empty());

    let result = resources_manager.remove::<ClusterDeployment>(testee.id).await?;
    assert_eq!(result, None);

    Ok(())
}
