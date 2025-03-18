pub mod create_cluster_configuration;
pub mod delete_cluster_configuration;
pub mod delete_cluster_deployment;
pub mod determine_cluster_peers;
pub mod determine_cluster_peer_states;
pub mod store_cluster_deployment;
pub mod list_deployed_clusters;

#[cfg(test)]
pub mod testing {
    use std::collections::HashSet;
    use opendut_types::cluster::{ClusterConfiguration, ClusterId, ClusterName};
    use crate::actions::testing::PeerFixture;
    use crate::resource::manager::ResourceManagerRef;

    pub struct ClusterFixture {
        pub id: ClusterId,
        pub configuration: ClusterConfiguration,
        pub peer_a: PeerFixture,
        pub peer_b: PeerFixture,
    }

    impl ClusterFixture {
        pub async fn create(resource_manager: ResourceManagerRef) -> anyhow::Result<ClusterFixture> {
            let peer_a = PeerFixture::new();
            let peer_b = PeerFixture::new();

            resource_manager.insert(peer_a.id, peer_a.descriptor.clone()).await?;
            resource_manager.insert(peer_b.id, peer_b.descriptor.clone()).await?;

            let cluster_id = ClusterId::random();
            let cluster_configuration = ClusterConfiguration {
                id: cluster_id,
                name: ClusterName::try_from(format!("Cluster-{cluster_id}"))?,
                leader: peer_a.id,
                devices: HashSet::from([peer_a.device_1, peer_a.device_2, peer_b.device_1]),
            };
            resource_manager.insert(cluster_id, cluster_configuration.clone()).await?;
            
            Ok(ClusterFixture {
                id: cluster_id,
                configuration: cluster_configuration,
                peer_a,
                peer_b,
            })

        }
    }
}
