use crate::resource::manager::ResourceManager;
use opendut_model::cluster::{ClusterDescriptor, ClusterDeployment};
use opendut_model::peer::PeerDescriptor;

#[tokio::test]
async fn should_persist_cluster_deployment() -> anyhow::Result<()> {
    let resource_manager = ResourceManager::new_in_memory();

    // Arrange
    let peer_descriptor = super::peer_descriptor::peer_descriptor()?;
    resource_manager.insert::<PeerDescriptor>(peer_descriptor.id, peer_descriptor.clone()).await?;
    let peer_descriptor_unused = super::peer_descriptor::peer_descriptor()?;
    resource_manager.insert::<PeerDescriptor>(peer_descriptor_unused.id, peer_descriptor_unused.clone()).await?;

    let cluster_descriptor = super::cluster_descriptor::cluster_descriptor(
        peer_descriptor.id,
        peer_descriptor.topology.devices.into_iter().map(|device| device.id).collect()
    )?;
    resource_manager.insert::<ClusterDescriptor>(cluster_descriptor.id, cluster_descriptor.clone()).await?;

    let testee = ClusterDeployment {
        id: cluster_descriptor.id,
    };

    let result = resource_manager.get::<ClusterDeployment>(testee.id).await?;
    assert!(result.is_none());
    let result = resource_manager.list::<ClusterDeployment>().await?;
    assert!(result.is_empty());

    // Act
    resource_manager.insert::<ClusterDeployment>(testee.id, testee.clone()).await?;

    // Assert
    let result = resource_manager.get::<ClusterDeployment>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));
    let result = resource_manager.list::<ClusterDeployment>().await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result.values().next(), Some(&testee));

    let result = resource_manager.remove::<ClusterDeployment>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));

    let result = resource_manager.get::<ClusterDeployment>(testee.id).await?;
    assert!(result.is_none());
    let result = resource_manager.list::<ClusterDeployment>().await?;
    assert!(result.is_empty());

    let result = resource_manager.remove::<ClusterDeployment>(testee.id).await?;
    assert_eq!(result, None);

    Ok(())
}
