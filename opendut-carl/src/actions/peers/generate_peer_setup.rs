use crate::persistence::error::PersistenceError;
use crate::resource::manager::ResourceManagerRef;
use crate::settings::vpn::Vpn;
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::UserId;
use opendut_types::peer::{PeerDescriptor, PeerId, PeerName, PeerSetup};
use opendut_types::util::net::{AuthConfig, Certificate};
use opendut_types::vpn::VpnPeerConfiguration;
use pem::Pem;
use tracing::{debug, error, info, warn};
use url::Url;

pub struct GeneratePeerSetupParams {
    pub resource_manager: ResourceManagerRef,
    pub peer: PeerId,
    pub carl_url: Url,
    pub ca: Pem,
    pub vpn: Vpn,
    pub oidc_registration_client: Option<RegistrationClientRef>,
    pub user_id: UserId,
}

#[derive(thiserror::Error, Debug)]
pub enum GeneratePeerSetupError {
    #[error("A PeerSetup for peer <{0}> could not be created, because a peer with that ID does not exist!")]
    PeerNotFound(PeerId),
    #[error("An error occurred while accessing persistence for creating a PeerSetup for peer <{peer_id}>")]
    Persistance { peer_id: PeerId, #[source] source: PersistenceError },
    #[error("An internal error occurred while creating a PeerSetup for peer '{peer_name}' <{peer_id}>:\n  {cause}")]
    Internal {
        peer_id: PeerId,
        peer_name: PeerName,
        cause: String
    }
}

#[tracing::instrument(skip(params), level="trace")]
pub async fn generate_peer_setup(params: GeneratePeerSetupParams) -> Result<PeerSetup, GeneratePeerSetupError> {

    async fn inner(params: GeneratePeerSetupParams) -> Result<PeerSetup, GeneratePeerSetupError> {

        let peer_id = params.peer;

        debug!("Generating PeerSetup for peer <{peer_id}>");

        let peer_descriptor = params.resource_manager.get::<PeerDescriptor>(peer_id).await
            .map_err(|source| GeneratePeerSetupError::Persistance { peer_id, source })?
            .ok_or(GeneratePeerSetupError::PeerNotFound(peer_id))?;

        let peer_name = peer_descriptor.name;

        let vpn_config = if let Vpn::Enabled { vpn_client } = &params.vpn {
            debug!("Retrieving VPN configuration for peer <{peer_id}>.");
            let vpn_config = vpn_client.generate_vpn_peer_configuration(params.peer).await
                .map_err(|cause| GeneratePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
            info!("Successfully retrieved vpn configuration for peer <{peer_id}>.");
            vpn_config
        }
        else {
            warn!("VPN is disabled. PeerSetup for peer '{peer_name}' <{peer_id}> will not contain any VPN information!");
            VpnPeerConfiguration::Disabled
        };

        let auth_config = match params.oidc_registration_client {
            None => {
                AuthConfig::Disabled
            }
            Some(registration_client) => {
                let resource_id = peer_id.into();
                debug!("Generating OIDC client for peer '{peer_name}' <{peer_id}>.");
                let issuer_url = registration_client.config.issuer_remote_url.clone();
                let client_credentials = registration_client.register_new_client_for_user(resource_id, params.user_id)
                    .await
                    .map_err(|cause| GeneratePeerSetupError::Internal { peer_id, peer_name: Clone::clone(&peer_name), cause: cause.to_string() })?;
                debug!("Successfully generated peer setup for peer '{peer_name}' <{peer_id}>. OIDC client_id='{}'.", client_credentials.client_id.clone().value());
                AuthConfig::from_credentials(issuer_url, client_credentials)
            }
        };

        Ok(PeerSetup {
            id: peer_id,
            carl: params.carl_url,
            ca: Certificate(params.ca),
            auth_config,
            vpn: vpn_config,
        })
    }

    inner(params).await
        .inspect_err(|err| error!("{err}"))
}
