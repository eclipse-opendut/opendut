use cfg_if::cfg_if;

pub mod broker;
pub mod cluster;
pub mod metadata;
pub mod peer;
pub mod observer;
#[cfg(feature="viper")]
pub mod viper;

cfg_if! {
    if #[cfg(any(feature = "client", feature = "wasm-client"))] {
        use std::fmt::Display;
        use tonic::codegen::http::uri::InvalidUri;
        use opendut_util::proto::ConversionError;

        #[derive(thiserror::Error, Debug)]
        pub enum ClientError<A>
        where
            A: Display
        {
            #[error("{0}")]
            TransportError(String),
            #[error("{0}")]
            InvalidRequest(String),
            #[error("{0}")]
            InvalidResponse(String),
            #[error("{0}")]
            UsageError(A),
        }

        impl <A> From<tonic::Status> for ClientError<A>
        where
            A: Display
        {
            fn from(status: tonic::Status) -> Self {
                match status.code() {
                    tonic::Code::InvalidArgument => {
                        Self::InvalidRequest(status.message().to_owned())
                    }
                    _ => {
                        Self::TransportError(status.message().to_owned())
                    }
                }
            }
        }

        impl <A> From<ConversionError> for ClientError<A>
        where
            A: Display
        {
            fn from(cause: ConversionError) -> Self {
                Self::InvalidResponse(cause.to_string())
            }
        }

        #[derive(thiserror::Error, Debug)]
        pub enum InitializationError {
            #[error("Invalid URI '{uri}': {cause}")]
            InvalidUri { uri: String, cause: InvalidUri },
            #[error("Expected https scheme. Given scheme: '{given_scheme}'")]
            ExpectedHttpsScheme { given_scheme: String },
            #[error("{message}: {cause}")]
            OidcConfiguration { message: String, cause: Box<dyn std::error::Error + Send + Sync> },
            #[error("{message}: {cause}")]
            TlsConfiguration { message: String, cause: Box<dyn std::error::Error + Send + Sync> },
            #[error("Error while connecting to CARL at '{address}': {cause}")]
            ConnectError { address: String, cause: Box<dyn std::error::Error + Send + Sync> },
        }

        pub trait ExtractOrClientError<A, B, E>
        where
            B: TryFrom<A>,
            B::Error: Display,
            E: Display
        {
            fn extract_or_client_error(self, field: impl Into<String> + Clone) -> Result<B, ClientError<E>>;
        }

        impl <A, B, E> ExtractOrClientError<A, B, E> for Option<A>
        where
            B: TryFrom<A>,
            B::Error: Display,
            E: Display
        {
            fn extract_or_client_error(self, field: impl Into<String> + Clone) -> Result<B, ClientError<E>> {
                self
                    .ok_or_else(|| ClientError::InvalidResponse(format!("Field '{}' not set", Clone::clone(&field).into())))
                    .and_then(|value| {
                        B::try_from(value)
                            .map_err(|cause| ClientError::InvalidResponse(format!("Field '{}' is not valid: {}", field.into(), cause)))
                    })
            }
        }

        macro_rules! extract {
            ($spec:expr) => {
                crate::carl::ExtractOrClientError::extract_or_client_error($spec, stringify!($spec))
            };
        }

        pub(crate) use extract;


        use std::pin::Pin;
        use tonic::codegen::tokio_stream::Stream;

        pub struct GrpcDownstream<T> {
            inner: Box<dyn Stream<Item=Result<T, tonic::Status>> + Send + Unpin>,
        }
        impl<T> GrpcDownstream<T> {
            pub async fn receive(&mut self) -> Result<Option<T>, tonic::Status> {
                match std::future::poll_fn(|cx| Pin::new(&mut *self.inner).poll_next(cx)).await {
                    Some(Ok(m)) => Ok(Some(m)),
                    Some(Err(e)) => Err(e),
                    None => Ok(None),
                }
            }
        }
        impl<T, S: Stream<Item=Result<T, tonic::Status>> + Send + Unpin + 'static> From<S> for GrpcDownstream<T> {
            fn from(value: S) -> Self {
                Self { inner: Box::new(value) }
            }
        }

    }
}

cfg_if! {
    if #[cfg(feature = "client")] {
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
    }
}

#[cfg(feature = "wasm-client")]
pub mod wasm {
    use tonic::codegen::InterceptedService;

    use opendut_auth::public::{AuthInterceptor, Authentication};

    use crate::carl::cluster::ClusterManager;
    use crate::carl::InitializationError;
    use crate::carl::metadata::MetadataProvider;
    use crate::carl::peer::PeersRegistrar;
    #[cfg(feature="viper")]
    use crate::carl::viper::TestManager;

    #[derive(Debug, Clone)]
    pub struct CarlClient {
        pub cluster: ClusterManager<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
        pub metadata: MetadataProvider<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
        pub peers: PeersRegistrar<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
        #[cfg(feature="viper")]
        pub viper: TestManager<InterceptedService<tonic_web_wasm_client::Client, AuthInterceptor>>,
    }

    impl CarlClient {
        pub async fn create(url: url::Url, auth: Authentication) -> Result<CarlClient, InitializationError> {
            let scheme = url.scheme();
            if scheme != "https" {
                return Err(InitializationError::ExpectedHttpsScheme { given_scheme: scheme.to_owned() });
            }

            let host = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(443_u16);

            let client = tonic_web_wasm_client::Client::new(format!("{scheme}://{host}:{port}"));
            let auth_interceptor = AuthInterceptor::new(auth);

            Ok(CarlClient {
                cluster: ClusterManager::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
                metadata: MetadataProvider::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
                peers: PeersRegistrar::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
                #[cfg(feature="viper")]
                viper: TestManager::with_interceptor(Clone::clone(&client), Clone::clone(&auth_interceptor)),
            })
        }
    }
}
