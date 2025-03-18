use crate::resource::persistence::database;
use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
use opendut_types::peer::PeerId;
use opendut_types::topology::DeviceId;
use std::collections::HashSet;
use crate::resource::manager::{ResourceManager, ResourceManagerRef};

#[tokio::test]
async fn should_persist_cluster_configuration_in_memory() -> anyhow::Result<()> {
    let resource_manager = ResourceManager::new_in_memory();
    should_persist_cluster_configuration(resource_manager).await
}

#[test_with::no_env(SKIP_DATABASE_CONTAINER_TESTS)]
#[tokio::test]
async fn should_persist_cluster_configuration_in_database() -> anyhow::Result<()> {
    let db = database::testing::spawn_and_connect_resource_manager().await?;
    should_persist_cluster_configuration(db.resource_manager).await
}

async fn should_persist_cluster_configuration(resource_manager: ResourceManagerRef) -> anyhow::Result<()> {

    let peer = super::peer_descriptor::peer_descriptor()?;
    resource_manager.insert(peer.id, peer.clone()).await?;

    let cluster_devices = peer.topology.devices.into_iter()
        .map(|device| device.id)
        .collect::<Vec<_>>();

    let testee = cluster_configuration(
        peer.id,
        cluster_devices.clone(),
    )?;

    let result = resource_manager.get::<ClusterConfiguration>(testee.id).await?;
    assert!(result.is_none());
    let result = resource_manager.list::<ClusterConfiguration>().await?;
    assert!(result.is_empty());

    resource_manager.insert(testee.id, testee.clone()).await?;

    let result = resource_manager.get::<ClusterConfiguration>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));
    let result = resource_manager.list::<ClusterConfiguration>().await?;
    assert_eq!(result.len(), 1);
    assert_eq!(result.first(), Some(&testee));

    let testee = {
        let mut testee = testee.clone();
        testee.devices.remove(&cluster_devices[0]);
        testee
    };
    resource_manager.insert(testee.id, testee.clone()).await?;

    let result = resource_manager.remove::<ClusterConfiguration>(testee.id).await?;
    assert_eq!(result, Some(testee.clone()));

    let result = resource_manager.get::<ClusterConfiguration>(testee.id).await?;
    assert!(result.is_none());
    let result = resource_manager.list::<ClusterConfiguration>().await?;
    assert!(result.is_empty());

    let result = resource_manager.remove::<ClusterConfiguration>(testee.id).await?;
    assert_eq!(result, None);

    Ok(())
}

pub fn cluster_configuration(leader_id: PeerId, devices: Vec<DeviceId>) -> anyhow::Result<ClusterConfiguration> {
    Ok(ClusterConfiguration {
        id: ClusterId::random(),
        name: ClusterName::try_from("cluster-name")?,
        leader: leader_id,
        devices: HashSet::from_iter(devices),
    })
}
