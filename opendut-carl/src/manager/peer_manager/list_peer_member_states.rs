use crate::manager::cluster_manager;
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;
use crate::resource::storage::ResourcesStorageApi;
use opendut_model::cluster::ClusterId;
use opendut_model::peer::state::PeerMemberState;
use opendut_model::peer::{PeerDescriptor, PeerId};
use opendut_model::topology::DeviceId;
use std::collections::HashMap;

impl Resources<'_> {
    pub fn list_peer_member_states(&self) -> Result<HashMap<PeerId, PeerMemberState>, ListPeerMemberStatesError> {
        let deployed_clusters = cluster_manager::internal::list_deployed_clusters(self)?;
        let deployed_devices = deployed_clusters.into_iter()
            .flat_map(|deployed_cluster| {
                let cluster_id = deployed_cluster.id;
                deployed_cluster.devices.into_iter().map(move |device_id| (device_id, cluster_id))
            })
            .collect::<HashMap<_, _>>();


        let all_peers = self.list::<PeerDescriptor>()?;

        let peer_member_states = all_peers.into_values()
            .map(|peer | {
                let blocked_devices = peer.topology.devices.into_iter()
                    .filter_map(|device| {
                        deployed_devices.get(&device.id).map(|cluster_id|
                            ClusterDevice { cluster_id: *cluster_id, device_id: device.id }
                        )
                    })
                    .collect::<Vec<_>>();

                match blocked_devices.as_slice() {
                    [] => (peer.id, PeerMemberState::Available),
                    [cluster_device, ..] => {
                        assert!(
                            blocked_devices.iter().all(|current_cluster_device| {
                                cluster_device.cluster_id == current_cluster_device.cluster_id
                            }),
                            "Expected all devices of the peer belonging to the same cluster! {blocked_devices:?}"
                        );
                        (peer.id, PeerMemberState::Blocked {by_cluster: cluster_device.cluster_id })
                    }
                }
            }).collect::<HashMap<_, _>>();
        Ok(peer_member_states)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ListPeerMemberStatesError {
    #[error("Failed to list peer member states.")]
    Persistence { #[from] source: PersistenceError },
}

#[derive(Debug)]
struct ClusterDevice {
    cluster_id: ClusterId,
    #[expect(unused)]
    device_id: DeviceId,  // only used for debug output in assertion
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manager::testing::ClusterFixture;
    use crate::resource::manager::ResourceManager;
    use opendut_model::cluster::ClusterDeployment;
    use std::collections::HashSet;
    use std::ops::Not;

    #[tokio::test]
    async fn test_list_blocked_peers() -> anyhow::Result<()> {
        // Arrange
        let resource_manager = ResourceManager::new_in_memory();
        let cluster_a = ClusterFixture::create(resource_manager.clone()).await?;
        let cluster_b = ClusterFixture::create(resource_manager.clone()).await?;
        resource_manager.insert(cluster_a.id, ClusterDeployment { id: cluster_a.id }).await?;

        // Act
        let peer_member_states = resource_manager.resources(async |resources|
            resources.list_peer_member_states()
        ).await??;

        // Assert
        let blocked_peers = peer_member_states.into_iter()
            .filter_map(|(peer_id, peer_member_state)| {
                if let PeerMemberState::Blocked { by_cluster } = peer_member_state {
                    Some((peer_id, by_cluster))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>();
        let blocked_peer_ids = blocked_peers.keys().collect::<HashSet<_>>();
        let deployed_cluster_ids = blocked_peers.values().collect::<HashSet<_>>();
        assert_eq!(blocked_peers.len(), 2);
        assert!(blocked_peer_ids.contains(&cluster_a.peer_a.id));
        assert!(blocked_peer_ids.contains(&cluster_a.peer_b.id));
        assert!(deployed_cluster_ids.contains(&cluster_b.id).not());

        Ok(())
    }
}
