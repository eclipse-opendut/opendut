use opendut_types::ShortName;
use opendut_types::peer::PeerDisplay;
use crate::resource::storage::ResourcesStorageApi;
use crate::settings::vpn::Vpn;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName};
use tracing::{debug, info, warn};
use opendut_types::cluster::ClusterId;
use opendut_types::peer::state::{PeerMemberState, PeerState};
use crate::manager::peer_manager::list_peer_member_states::ListPeerMemberStatesError;
use crate::resource::api::resources::Resources;
use crate::resource::persistence::error::PersistenceError;

pub struct DeletePeerDescriptorParams {
    pub vpn: Vpn,
    pub peer: PeerId,
    pub oidc_registration_client: Option<RegistrationClientRef>,
}

impl Resources<'_> {
    #[tracing::instrument(skip_all, level="trace")]
    pub async fn delete_peer_descriptor(&mut self, params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

        let peer_id = params.peer;

        let peer_member_states = self.list_peer_member_states()
            .map_err(|source| match source {
                ListPeerMemberStatesError::Persistence { source } => {
                    DeletePeerDescriptorError::Persistence { peer_id, peer_name: None, source }
                },
            })?;
        let peer_member_state = peer_member_states.get(&peer_id);

        if let Some(PeerMemberState::Blocked { by_cluster }) = peer_member_state {
            Err(DeletePeerDescriptorError::ClusterDeploymentExists { peer_id, cluster_id: *by_cluster })
        } else {
            debug!("Deleting peer descriptor of peer <{peer_id}>.");

            let peer_descriptor = self.remove::<PeerDescriptor>(peer_id)
                .map_err(|source| DeletePeerDescriptorError::Persistence { peer_id, peer_name: None, source })?
                .ok_or_else(|| DeletePeerDescriptorError::PeerNotFound { peer_id })?;

            let peer_name = &peer_descriptor.name;

            if let Some(registration_client) = params.oidc_registration_client {
                let resource_id = peer_id.into();
                debug!("Deleting OIDC client for peer '{peer_name}' <{peer_id}>.");
                let deleted_clients = registration_client.delete_client_by_resource_id(resource_id)
                    .await
                    .map_err(|source| DeletePeerDescriptorError::AuthRegistration { peer_id, peer_name: peer_name.clone(), source })?;
                let deleted_client_ids =  deleted_clients.value().into_iter().map(|client| client.client_id).collect::<Vec<String>>();
                debug!("Successfully deleted oidc clients for peer '{peer_name}' <{peer_id}>. OIDC client_ids='{}'.", deleted_client_ids.join(","));
            }

            if let Vpn::Enabled { vpn_client } = params.vpn {
                debug!("Deleting vpn peer <{peer_id}>.");
                vpn_client.delete_peer(peer_id)
                    .await
                    .map_err(|source| DeletePeerDescriptorError::VpnClient {
                        peer_id,
                        peer_name: peer_name.clone(),
                        source,
                    })?;
                info!("Successfully deleted VPN peer <{peer_id}>.");
            } else {
                warn!("VPN disabled. Skipping VPN peer deletion!");
            }

            info!("Successfully deleted peer descriptor of '{peer_name}' <{peer_id}>.");

            Ok(peer_descriptor)
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeletePeerDescriptorError {
    #[error("Peer <{peer_id}> could not be deleted, because a peer with that id does not exist!")]
    PeerNotFound {
        peer_id: PeerId,
    },
    #[error("Peer <{peer_id}> could not be deleted, because a cluster deployment <{cluster_id}> using this peer still exists!")]
    ClusterDeploymentExists {
        peer_id: PeerId,
        cluster_id: ClusterId,
    },
    #[error(
        "Peer {peer} cannot be deleted in state '{actual_state}'! A peer can be deleted when: {required_states}",
        peer = PeerDisplay::new(&Some(peer_name.to_owned()), peer_id),
        actual_state = actual_state.short_name(),
        required_states = PeerState::short_names_joined(required_states),
    )]
    IllegalPeerState {
        peer_id: PeerId,
        peer_name: PeerName,
        actual_state: PeerState,
        required_states: Vec<PeerState>,
    },
    #[error("Error when accessing persistence while deleting peer descriptor for peer {peer}", peer=PeerDisplay::new(peer_name, peer_id))]
    Persistence {
        peer_id: PeerId,
        peer_name: Option<PeerName>,
        #[source] source: PersistenceError,
    },
    #[error("Error when removing registration in authentication provider while deleting peer descriptor for peer {peer}", peer=PeerDisplay::new(&Some(peer_name.to_owned()), peer_id))]
    AuthRegistration {
        peer_id: PeerId,
        peer_name: PeerName,
        #[source] source: opendut_auth::registration::client::RegistrationClientError,
    },
    #[error("Error when deleting peer in VPN management while deleting peer descriptor for peer {peer}", peer=PeerDisplay::new(&Some(peer_name.to_owned()), peer_id))]
    VpnClient {
        peer_id: PeerId,
        peer_name: PeerName,
        #[source] source: opendut_vpn::DeletePeerError,
    },
}

#[cfg(test)]
mod tests {
    use opendut_types::cluster::ClusterDeployment;
    use crate::manager::testing::ClusterFixture;
    use crate::resource::manager::ResourceManager;
    use super::*;

    #[tokio::test]
    async fn block_deletion_of_peer_descriptor_if_a_cluster_with_this_peer_is_still_deployed() -> anyhow::Result<()> {
        // Arrange
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;
        resource_manager.insert(cluster.id, ClusterDeployment { id: cluster.id }).await?;

        // Act
        let delete_peer_descriptor_params = DeletePeerDescriptorParams { vpn: Vpn::Disabled, peer: cluster.peer_a.id, oidc_registration_client: None };
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_peer_descriptor(delete_peer_descriptor_params).await
        ).await?;

        // Assert
        let Err(DeletePeerDescriptorError::ClusterDeploymentExists { peer_id, cluster_id }) = result
        else { panic!("Result is not an error of ClusterDeploymentExists.") };

        assert_eq!(peer_id, cluster.peer_a.id);
        assert_eq!(cluster_id, cluster.id);

        Ok(())
    }

    #[tokio::test]
    async fn delete_cluster_descriptor_when_cluster_is_not_deployed() -> anyhow::Result<()> {
        // Arrange
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;

        // Act
        let delete_peer_descriptor_params = DeletePeerDescriptorParams { vpn: Vpn::Disabled, peer: cluster.peer_a.id, oidc_registration_client: None };
        let result = resource_manager.resources_mut(async |resources|
            resources.delete_peer_descriptor(delete_peer_descriptor_params).await
        ).await??;

        // Assert
        assert_eq!(result, cluster.peer_a.descriptor);

        Ok(())
    }
}
