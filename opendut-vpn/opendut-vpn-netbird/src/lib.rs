use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Url;
pub use netbird::token::Token as NetbirdToken;
use opendut_types::cluster::ClusterId;
use opendut_types::peer::PeerId;
use opendut_types::vpn::{HttpsOnly, VpnPeerConfig};
use opendut_vpn::{CreateClusterError, CreatePeerError, DeleteClusterError, DeletePeerError, GetOrCreateConfigurationError, VpnManagementClient};
use crate::client::Client;
use crate::netbird::error::{CreateClientError, GetGroupError, GetRulesError, RequestError};
use crate::netbird::rules::RuleName;

mod client;
mod routes;
mod netbird;

pub struct DefaultVpnManagementClient {
    inner: Client
}

impl DefaultVpnManagementClient {

    pub fn create(base_url: Url, token: NetbirdToken, https_only: HttpsOnly) -> Result<Self, CreateClientError> {
        Ok(Self {
            inner: Client::create(
                base_url,
                token,
                https_only,
            )?
        })
    }
}

#[async_trait]
impl VpnManagementClient for DefaultVpnManagementClient {

    async fn create_cluster(&self, cluster_id: ClusterId, peers: &[PeerId]) -> Result<(), CreateClusterError> {
        match self.delete_cluster(cluster_id).await {
            Ok(_) => log::debug!("Deleted a previous cluster with ID <{cluster_id}> before creating the new cluster."),
            Err(cause) => match cause {
                DeleteClusterError::NotFound { cluster_id, message } => log::trace!("Did not need to delete a previous cluster with ID <{cluster_id}> before creating the new cluster. ({message})"),
                DeleteClusterError::DeletionFailure { cluster_id, error } => {
                    return Err(CreateClusterError::CreationFailure { cluster_id, error: anyhow!("Failure while deleting a previous cluster with ID <{cluster_id}> before creating the new cluster: {error}").into() });
                }
            }
        };

        let netbird_peers: Vec<netbird::PeerId> = {
            let mut netbird_peers = vec![];
            for peer_id in peers {
                let group = self.inner.get_netbird_group(&(*peer_id).into()).await
                    .map_err(|error| CreateClusterError::PeerResolutionFailure { peer_id: *peer_id, cluster_id, error: error.into() })?;
                let peer = group.peers.into_iter().next()
                    .ok_or(CreateClusterError::PeerResolutionFailure { peer_id: *peer_id, cluster_id, error: anyhow!("Self-Group does not contain expected peer!").into() })?;
                netbird_peers.push(peer.id);
            }
            netbird_peers
        };

        let group = self.inner.create_netbird_group(cluster_id.into(), netbird_peers).await
            .map_err(|error| CreateClusterError::CreationFailure { cluster_id, error: error.into() })?;


        self.inner.create_netbird_self_access_control_rule(group, cluster_id.into()).await
            .map_err(|error| CreateClusterError::AccessControlRuleCreationFailure { cluster_id, error: error.into() })?;

        Ok(())
    }

    async fn delete_cluster(&self, cluster_id: ClusterId) -> Result<(), DeleteClusterError> {
        // delete cluster rule (access control)
        let rule_name = RuleName::Cluster(cluster_id);
        match self.inner.get_netbird_rule(&rule_name).await {
            Ok(rule) => {
                match self.inner.delete_netbird_rule(&rule.id).await {
                    Ok(_) => log::debug!("Deleted NetBird rule with name '{}' and NetBird Rule ID '{}'.", rule.name, rule.id.0),
                    Err(cause) => return match cause {
                        RequestError::IllegalStatus(error) => {
                            if let Some(http::StatusCode::NOT_FOUND) = error.status() {
                                Err(DeleteClusterError::NotFound { cluster_id, message: format!("Received '404 Not Found' when deleting rule for cluster <{cluster_id}> with NetBird rule ID <{netbird_rule}>.", netbird_rule = rule.id.0) })
                            } else {
                                Err(DeleteClusterError::DeletionFailure { cluster_id, error: error.into() })
                            }
                        }
                        RequestError::IllegalRequest(error, code) => {
                            Err(DeleteClusterError::DeletionFailure {
                                cluster_id,
                                error: anyhow!("Received status code '{code}' when deleting cluster <{cluster_id}> with NetBird rule ID <{netbird_rule}>:\n  {error}", code=code, cluster_id=cluster_id, netbird_rule=rule.id.0, error=error).into(),
                            })
                        }
                        other => Err(DeleteClusterError::DeletionFailure { cluster_id, error: other.into() }),
                    }
                }
            }
            Err(GetRulesError::RuleNotFound { .. }) => {
                // No rule found, so no need to delete it.
            }
            Err(cause) => {
                return Err(DeleteClusterError::DeletionFailure { cluster_id, error: anyhow!("Failed to get cluster rule '{rule_name}' to be deleted.\n {cause}").into() });
            }
        };

        let group_name = netbird::GroupName::from(cluster_id);
        match self.inner.get_netbird_group(&group_name).await {
            Ok(group) => {
                match self.inner.delete_netbird_group(&group.id).await {
                    Ok(_) => {
                        log::debug!("Deleted NetBird group with name '{}' and NetBird Group ID '{}'.", group.name, group.id.0) ;
                        Ok(())
                    },
                    Err(cause) => match cause {
                        RequestError::IllegalStatus(error) => {
                            if let Some(http::StatusCode::NOT_FOUND) = error.status() {
                                Err(DeleteClusterError::NotFound { cluster_id, message: format!("Received '404 Not Found' when deleting group for cluster <{cluster_id}> with NetBird group ID <{netbird_group}>.", netbird_group = group.id.0) })
                            } else {
                                Err(DeleteClusterError::DeletionFailure { cluster_id, error: error.into() }) //TODO logging of this doesn't show the HTTP body, making e.g. 400 Bad Request errors difficult to debug
                            }
                        }
                        other => Err(DeleteClusterError::DeletionFailure { cluster_id, error: other.into() }),
                    }
                }

            }
            Err(GetGroupError::GroupNotFound { .. }) => {
                // No group found, so no need to delete it.
                Ok(())
            }
            Err(cause) => {
                Err(DeleteClusterError::DeletionFailure { cluster_id, error: anyhow!("Failed to get cluster group '{group_name}' to be deleted.\n {cause}").into() })
            }
        }
    }

    async fn create_peer(&self, peer_id: PeerId) -> Result<(), CreatePeerError> {
        let peers = vec![]; //Peer self-group does not have peers
        self.inner.create_netbird_group(peer_id.into(), peers).await
            .map_err(|error| CreatePeerError::CreationFailure { peer_id, error: error.into() })?;
        Ok(())
    }

    async fn delete_peer(&self, peer_id: PeerId) -> Result<(), DeletePeerError> {
        let self_group = self.inner.get_netbird_group(&netbird::GroupName::from(peer_id)).await
            .map_err(|error| DeletePeerError::ResolutionFailure { peer_id, error: error.into() })?;

        if let Some(peer_info) = self_group.peers.first() {
            self.inner.delete_netbird_peer(&peer_info.id)
                .await
                .map_err(|error| DeletePeerError::DeletionFailure { peer_id, error: error.into() })?;
        }

        self.inner.delete_netbird_group(&self_group.id)
            .await
            .map_err(|error| DeletePeerError::DeletionFailure { peer_id, error: error.into() })?;

        Ok(())
    }

    async fn create_peer_configuration(&self, peer_id: PeerId) -> Result<VpnPeerConfig, GetOrCreateConfigurationError> {
        let setup_keys = self.inner.list_netbird_setup_keys().await
            .map_err(|error| GetOrCreateConfigurationError::QueryConfigurationsFailure { error: error.into() })?;

        let maybe_setup_key = setup_keys.into_iter()
            .find(|setup_key| setup_key.name == netbird::setup_key::name_format(peer_id));

        let setup_key = match maybe_setup_key {
            None => {
                self.inner.create_netbird_setup_key(peer_id).await
                    .map_err(|error| GetOrCreateConfigurationError::CreationFailure { peer_id, error: error.into() })?
            }
            Some(setup_key) => setup_key,
        };

        Ok(VpnPeerConfig::Netbird {
            management_url: Clone::clone(self.inner.base_url()),
            setup_key: opendut_types::vpn::netbird::SetupKey::from(setup_key.key),
        })
    }
}
