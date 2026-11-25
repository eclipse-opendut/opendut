#![allow(deprecated)]

use std::collections::HashSet;
use tracing::{debug, info};
use opendut_auth::registration::client::RegistrationClientRef;
use opendut_auth::registration::resources::ResourceHomeUrl;
use opendut_auth::types::{SCOPE_ADMIN_API, SCOPE_EDGE_API};
use opendut_model::peer::PeerDescriptor;
use crate::resource::manager::ResourceManagerRef;


pub async fn run(_resource_manager: ResourceManagerRef) -> anyhow::Result<()> {
    debug!("Running migrations...");

    //insert migrations here

    debug!("Migrations complete.");
    Ok(())
}

pub async fn migrate_oidc_client_scopes(
    resource_manager: ResourceManagerRef,
    registration_client: Option<RegistrationClientRef>,
) -> anyhow::Result<()> {
    let Some(registration_client) = registration_client else {
        return Ok(()); // OIDC disabled
    };

    // 1. Collect all known PeerIds from CARL persistence
    let peer_ids: HashSet<String> = resource_manager
        .list::<PeerDescriptor>()
        .await?
        .into_keys()
        .map(|peer_id| peer_id.uuid.to_string())
        .collect();

    // 2. Fetch all clients once (already includes defaultClientScopes)
    let all_clients = registration_client.list_clients().await?;

    // 3. Filter dynamic clients created by CARL (base_url starts with CARL's /resources/ URL)
    let resource_base_url = ResourceHomeUrl::new(
        registration_client.config.client_home_base_url.value().join("/resources/")?
    );
    let dynamic_clients = all_clients.filter_carl_clients(&resource_base_url);

    for client in dynamic_clients {
        let is_edgar = client.base_url.as_ref().is_some_and(|url| {
            peer_ids.iter().any(|peer_id| url.contains(peer_id))
        });

        let required_scope = if is_edgar {
            SCOPE_EDGE_API
        } else {
            SCOPE_ADMIN_API // Dynamic clients not matching a PeerId are CLEO setups
        };

        let already_assigned = client.default_client_scopes
            .as_ref()
            .is_some_and(|scopes| scopes.iter().any(|s| s == required_scope));

        if !already_assigned {
            registration_client.assign_scope_to_client(&client.id, required_scope).await?;
            info!("Migrated OIDC client '{}' with scope '{required_scope}'.", client.client_id);
        }
    }

    Ok(())
}
