use tracing::{debug, info};
use tonic::transport::Identity;
use tower::ServiceBuilder;

use opendut_auth::confidential::client::ConfidentialClient;
use opendut_auth::confidential::tonic_service::TonicAuthenticationService;
use opendut_util::pem::{join_pem_objects, Pem, ClientAuth};

use crate::carl::cluster::ClusterManager;
use crate::carl::metadata::MetadataProvider;
use crate::carl::peer::PeersRegistrar;
use crate::carl::broker::PeerMessagingBroker;
use crate::carl::observer::ObserverMessagingBroker;
use crate::carl::secret::SecretManager;
#[cfg(feature="viper")]
use crate::carl::viper::TestManager;

use crate::proto::services::cluster_manager::cluster_manager_client::ClusterManagerClient;
use crate::proto::services::metadata_provider::metadata_provider_client::MetadataProviderClient;
use crate::proto::services::peer_manager::peer_manager_client::PeerManagerClient;
use crate::proto::services::peer_messaging_broker::peer_messaging_broker_client::PeerMessagingBrokerClient;
use crate::proto::services::observer_messaging_broker::observer_messaging_broker_client::ObserverMessagingBrokerClient;
use crate::proto::services::secret_manager::secret_manager_client::SecretManagerClient;
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
    pub secret: SecretManager<TonicAuthenticationService>,
    #[cfg(feature="viper")]
    pub viper: TestManager<TonicAuthenticationService>,
}

impl CarlClient {

    #[tracing::instrument(name="carl_client_create", skip_all)]
    pub async fn create(
        host: &str,
        port: u16,
        ca_certs: &[Pem],
        client_auth: &ClientAuth,
        domain_name_override: &Option<String>,
        settings: &config::Config,
    ) -> Result<CarlClient, InitializationError> {

        let address = format!("https://{host}:{port}");
        let certs = ca_certs.iter().map(|cert| tonic::transport::Certificate::from_pem(cert.to_string())).collect::<Vec<_>>();
        debug!("Loaded {} CA certificates for TLS connection to CARL.", certs.len());

        let tls_config = {
            let mut config = tonic::transport::ClientTlsConfig::new()
                .ca_certificates(certs);

            if let ClientAuth::Enabled { certs, key } = client_auth {
                debug!("Configuring mTLS client authentication...");
                let client_certificate = certs.first().cloned().ok_or(InitializationError::TlsClientConfiguration {
                    message: String::from("No client certificate found for mTLS client authentication.")
                })?;
                let name = pem::read_certificate_subject(&client_certificate);
                debug!("Using CARL client certificate with subject '<{name:?}>'.");
                let client_certs = join_pem_objects(certs);
                config = config.identity(Identity::from_pem(client_certs, key.to_string()));
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
            secret: SecretManager::new(SecretManagerClient::new(Clone::clone(&auth_service))),
            #[cfg(feature="viper")]
            viper: TestManager::new(TestManagerClient::new(Clone::clone(&auth_service))),
        })
    }
}

use tokio::sync::mpsc;
use tokio::sync::mpsc::error::SendError;
use opendut_util::pem;
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
