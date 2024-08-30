use crate::resources::manager::ResourcesManagerRef;
use crate::resources::storage::ResourcesStorageApi;
use crate::vpn::Vpn;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_carl_api::carl::peer::DeletePeerDescriptorError;
use opendut_types::peer::{PeerDescriptor, PeerId};
use tracing::{debug, error, info, warn};

pub struct DeletePeerDescriptorParams {
    pub resources_manager: ResourcesManagerRef,
    pub vpn: Vpn,
    pub peer: PeerId,
    pub oidc_registration_client: Option<RegistrationClientRef>,
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn delete_peer_descriptor(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

    async fn inner(params: DeletePeerDescriptorParams) -> Result<PeerDescriptor, DeletePeerDescriptorError> {

        let peer_id = params.peer;
        let resources_manager = params.resources_manager;

        debug!("Deleting peer descriptor of peer <{peer_id}>.");

        let peer_descriptor = resources_manager.resources_mut(|resources| {

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

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
