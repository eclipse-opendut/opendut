use crate::resource::manager::ResourceManagerRef;
use crate::resource::storage::ResourcesStorageApi;
use crate::settings::vpn::Vpn;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_carl_api::carl::peer::DeletePeerDescriptorError;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info, warn};
use opendut_types::peer::state::PeerMemberState;
use crate::manager::peer_manager;
use crate::manager::peer_manager::ListPeerMemberStatesParams;

pub struct DeletePeerDescriptorParams {
    pub resource_manager: ResourceManagerRef,
    pub vpn: Vpn,
    pub peer: PeerId,
    pub oidc_registration_client: Option<RegistrationClientRef>,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_peer_descriptor(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

    async fn inner(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

        let peer_id = params.peer;
        let resource_manager = params.resource_manager;

        let peer_member_states = peer_manager::list_peer_member_states(ListPeerMemberStatesParams { resource_manager: resource_manager.clone() }).await
            .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: None, cause: cause.to_string() })?;  // only persistence error possible
        let peer_member_state = peer_member_states.get(&peer_id);
        
        if let Some(PeerMemberState::Blocked { by_cluster }) = peer_member_state {
            Err(DeletePeerDescriptorError::ClusterDeploymentExists { peer_id, cluster_id: *by_cluster })
        } else {
            debug!("Deleting peer descriptor of peer <{peer_id}>.");

            let peer_descriptor = resource_manager.resources_mut(async |resources| {

                let peer_descriptor = resources.remove::<PeerDescriptor>(peer_id)
                    .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: None, cause: cause.to_string() })?
                    .ok_or_else(|| DeletePeerDescriptorError::PeerNotFound { peer_id })?;

                Ok(peer_descriptor)
            }).await
                .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: None, cause: cause.to_string() })??;

            let peer_name = &peer_descriptor.name;

            if let Some(registration_client) = params.oidc_registration_client {
                let resource_id = peer_id.into();
                debug!("Deleting OIDC client for peer '{peer_name}' <{peer_id}>.");
                let deleted_clients = registration_client.delete_client_by_resource_id(resource_id)
                    .await
                    .map_err(|cause| DeletePeerDescriptorError::Internal { peer_id, peer_name: Some(peer_name.clone()), cause: cause.to_string() })?;
                let deleted_client_ids =  deleted_clients.value().into_iter().map(|client| client.client_id).collect::<Vec<String>>();
                debug!("Successfully deleted oidc clients for peer '{peer_name}' <{peer_id}>. OIDC client_ids='{}'.", deleted_client_ids.join(","));
            };

            if let Vpn::Enabled { vpn_client } = params.vpn {
                debug!("Deleting vpn peer <{peer_id}>.");
                vpn_client.delete_peer(peer_id)
                    .await
                    .map_err(|cause| DeletePeerDescriptorError::Internal {
                        peer_id,
                        peer_name: Some(peer_name.clone()),
                        cause: cause.to_string()
                    })?;
                info!("Successfully deleted VPN peer <{peer_id}>.");
            } else {
                warn!("VPN disabled. Skipping VPN peer deletion!");
            }

            info!("Successfully deleted peer descriptor of '{peer_name}' <{peer_id}>.");

            Ok(peer_descriptor)
        }

    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
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
        let delete_peer_descriptor_params = DeletePeerDescriptorParams { resource_manager, vpn: Vpn::Disabled, peer: cluster.peer_a.id, oidc_registration_client: None };
        let result = delete_peer_descriptor(delete_peer_descriptor_params).await;

        // Assert
        let expected_error = Err(DeletePeerDescriptorError::ClusterDeploymentExists { peer_id: cluster.peer_a.id, cluster_id: cluster.id });
        assert_eq!(expected_error, result);
        Ok(())
    }

    #[tokio::test]
    async fn delete_cluster_configuration_when_cluster_is_not_deployed() -> anyhow::Result<()> {
        // Arrange
        let resource_manager = ResourceManager::new_in_memory();
        let cluster = ClusterFixture::create(resource_manager.clone()).await?;

        // Act
        let delete_peer_descriptor_params = DeletePeerDescriptorParams { resource_manager, vpn: Vpn::Disabled, peer: cluster.peer_a.id, oidc_registration_client: None };
        let result = delete_peer_descriptor(delete_peer_descriptor_params).await;

        // Assert
        let expected_result = Ok(cluster.peer_a.descriptor);
        assert_eq!(expected_result, result);

        Ok(())
    }
}
