use std::net::IpAddr;
use async_trait::async_trait;
use opendut_types::cluster::ClusterId;

use opendut_types::peer::PeerId;
use opendut_types::vpn::VpnPeerConfig;

#[async_trait]
pub trait VpnManagementClient {

    async fn create_cluster(&self, cluster_id: ClusterId, peers: &Vec<PeerId>) -> Result<(), CreateClusterError>;

    async fn delete_cluster(&self, cluster_id: ClusterId) -> Result<(), DeleteClusterError>;

    async fn create_peer(&self, peer_id: PeerId) -> Result<(), CreatePeerError>;

    async fn delete_peer(&self, peer_id: PeerId) -> Result<(), DeletePeerError>;

    async fn get_or_create_configuration(&self, peer_id: PeerId) -> Result<VpnPeerConfig, GetOrCreateConfigurationError>;

    async fn get_peer_vpn_address(&self, peer_id: PeerId) -> Result<IpAddr, GetPeerVpnAddressError>;
}

#[derive(thiserror::Error, Debug)]
pub enum CreateClusterError {
    #[error("Peer <{peer_id}> of cluster <{cluster_id}> could not be resolved:\n  {error}")]
    PeerResolutionFailure {
        peer_id: PeerId,
        cluster_id: ClusterId,
        error: Box<dyn std::error::Error>,
    },
    #[error("An error occurred while creating cluster <{cluster_id}>:\n  {error}")]
    CreationFailure {
        cluster_id: ClusterId,
        error: Box<dyn std::error::Error>
    },
    #[error("An error occurred while creating access control rule for cluster <{cluster_id}>:\n  {error}")]
    AccessControlRuleCreationFailure {
        cluster_id: ClusterId,
        error: Box<dyn std::error::Error>
    }
}

#[derive(thiserror::Error, Debug)]
pub enum DeleteClusterError {
    #[error("No cluster <{cluster_id}> could be found: {message}")]
    NotFound {
        cluster_id: ClusterId,
        message: String,
    },
    #[error("An error occurred while deleting cluster <{cluster_id}>:\n  {error}")]
    DeletionFailure {
        cluster_id: ClusterId,
        error: Box<dyn std::error::Error>
    },
}

#[derive(thiserror::Error, Debug)]
pub enum CreatePeerError {
    #[error("An error occurred while creating peer <{peer_id}>:\n  {error}")]
    CreationFailure {
        peer_id: PeerId,
        error: Box<dyn std::error::Error>
    }
}

#[derive(thiserror::Error, Debug)]
pub enum GetPeerVpnAddressError {
    #[error("Peer <{peer_id}> could not be resolved:\n  {error}")]
    ResolutionFailure {
        peer_id: PeerId,
        error: Box<dyn std::error::Error>
    },
}

#[derive(thiserror::Error, Debug)]
pub enum DeletePeerError {
    #[error("Peer <{peer_id}> could not be resolved:\n  {error}")]
    ResolutionFailure {
        peer_id: PeerId,
        error: Box<dyn std::error::Error>
    },
    #[error("An error occurred while deleting peer <{peer_id}>:\n  {error}")]
    DeletionFailure {
        peer_id: PeerId,
        error: Box<dyn std::error::Error>
    },
}

#[derive(thiserror::Error, Debug)]
pub enum GetOrCreateConfigurationError {
    #[error("Failed to query configurations:\n  {error}")]
    QueryConfigurationsFailure {
        error: Box<dyn std::error::Error>
    },
    #[error("A configuration for peer <{peer_id}> could not be created:\n  {error}")]
    CreationFailure {
        peer_id: PeerId,
        error: Box<dyn std::error::Error>
    },
}
