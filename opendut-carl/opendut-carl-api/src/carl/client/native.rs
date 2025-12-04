use tracing::{debug, info};
use tonic::transport::Identity;
use tower::ServiceBuilder;

use opendut_auth::confidential::client::ConfidentialClient;
use opendut_auth::confidential::tonic_service::TonicAuthenticationService;
use opendut_util::pem::Pem;
use opendut_util::client_auth::ClientAuth;

use crate::carl::cluster::ClusterManager;
use crate::carl::metadata::MetadataProvider;
use crate::carl::peer::PeersRegistrar;
use crate::carl::broker::PeerMessagingBroker;
use crate::carl::observer::ObserverMessagingBroker;
#[cfg(feature="viper")]
use crate::carl::viper::TestManager;

use crate::proto::services::cluster_manager::cluster_manager_client::ClusterManagerClient;
use crate::proto::services::metadata_provider::metadata_provider_client::MetadataProviderClient;
use crate::proto::services::peer_manager::peer_manager_client::PeerManagerClient;
use crate::proto::services::peer_messaging_broker::peer_messaging_broker_client::PeerMessagingBrokerClient;
use crate::proto::services::observer_messaging_broker::observer_messaging_broker_client::ObserverMessagingBrokerClient;
#[cfg(feature="viper")]
use crate::proto::services::test_manager::test_manager_client::TestManagerClient;

use super::InitializationError;

#[derive(Clone)]
pub struct CarlClient {
    pub broker: PeerMessagingBroker<TonicAuthenticationService>,
    pub cluster: ClusterManager<TonicAuthenticationService>,
    pub metadata: MetadataProvider<TonicAuthenticationService>,
    pub peers: PeersRegistrar<TonicAuthenticationService>,
    pub observer: ObserverMessagingBroker<TonicAuthenticationService>,
    #[cfg(feature="viper")]
    pub viper: TestManager<TonicAuthenticationService>,
}

impl CarlClient {

    pub async fn create(
        host: &str,
        port: u16,
        ca_cert: &Pem,
        client_auth: &ClientAuth,
        domain_name_override: &Option<String>,
        settings: &config::Config,
    ) -> Result<CarlClient, InitializationError> {

        let address = format!("https://{host}:{port}");

        let tls_config = {
            let mut config = tonic::transport::ClientTlsConfig::new()
                .ca_certificate(tonic::transport::Certificate::from_pem(ca_cert.to_string()));

            if let ClientAuth::Enabled { cert, key } = client_auth {
                debug!("Configuring mTLS client authentication...");
                config = config.identity(Identity::from_pem(cert.to_string(), key.to_string()));
            }

            if let Some(domain_name_override) = domain_name_override {
                debug!("Using override for verified domain name of '{domain_name_override}'.");
                config = config.domain_name(domain_name_override);
            }
            config
        };

        let endpoint = tonic::transport::Channel::from_shared(address.clone())
            .map_err(|cause| InitializationError::InvalidUri { uri: address.clone(), cause })?
            .tls_config(tls_config)
            .map_err(|cause| InitializationError::TlsConfiguration { message: String::from("Failed to initialize secure channel with specified TLS configuration"), cause: cause.into() })?;

        let oidc_client = ConfidentialClient::from_settings(settings).await
            .map_err(|cause| InitializationError::OidcConfiguration { message: String::from("Failed to initialize OIDC authentication manager"), cause: cause.into() })?;

        if let Some(oidc_client) = &oidc_client {
            oidc_client.check_login().await
                .map_err(|cause| InitializationError::ConnectError { address: address.clone(), cause: cause.into() })?;
        }
        debug!("Set up endpoint for connection to CARL at '{address}'.");

        let channel = endpoint.connect().await
            .map_err(|cause| InitializationError::ConnectError { address: address.clone(), cause: cause.into() })?;
        info!("Connected to CARL at '{address}'.");

        let auth_service = ServiceBuilder::new()
            .layer_fn(|channel| TonicAuthenticationService::new(channel, oidc_client.clone()))
            .service(channel);

        Ok(CarlClient {
            broker: PeerMessagingBroker::new(PeerMessagingBrokerClient::new(Clone::clone(&auth_service))),
            cluster: ClusterManager::new(ClusterManagerClient::new(Clone::clone(&auth_service))),
            metadata: MetadataProvider::new(MetadataProviderClient::new(Clone::clone(&auth_service))),
            peers: PeersRegistrar::new(PeerManagerClient::new(Clone::clone(&auth_service))),
            observer: ObserverMessagingBroker::new(ObserverMessagingBrokerClient::new(Clone::clone(&auth_service))),
            #[cfg(feature="viper")]
            viper: TestManager::new(TestManagerClient::new(Clone::clone(&auth_service))),
        })
    }
}

use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use crate::proto::services::peer_messaging_broker;

#[derive(Debug, Clone)]
pub struct GrpcUpstream {
    inner: mpsc::Sender<peer_messaging_broker::Upstream>,
}
impl GrpcUpstream {
    pub async fn send<T: Into<peer_messaging_broker::Upstream>>(&self, message: T) -> Result<(), SendError<peer_messaging_broker::Upstream>> {
        self.inner.send(message.into()).await
    }
}
impl From<mpsc::Sender<peer_messaging_broker::Upstream>> for GrpcUpstream {
    fn from(value: mpsc::Sender<peer_messaging_broker::Upstream>) -> Self {
        Self { inner: value }
    }
}
