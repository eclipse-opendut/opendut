use cfg_if::cfg_if;

pub mod broker;
pub mod cluster;
pub mod metadata;
pub mod peer;

cfg_if! {
    if #[cfg(any(feature = "client", feature = "wasm-client"))] {
        use std::fmt::Display;
        use tonic::codegen::http::uri::InvalidUri;
        use opendut_types::proto::ConversionError;

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
            #[error("{message}: {cause}")]
            TlsConfiguration { message: String, cause: Box<dyn std::error::Error + Send + Sync> },
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
    }
}

cfg_if! {
    if #[cfg(feature = "client")] {
        use crate::carl::cluster::ClusterManager;
        use crate::carl::metadata::MetadataProvider;
        use crate::carl::peer::PeersRegistrar;
        use crate::carl::broker::PeerMessagingBroker;

        use crate::proto::services::cluster_manager::cluster_manager_client::ClusterManagerClient;
        use crate::proto::services::metadata_provider::metadata_provider_client::MetadataProviderClient;
        use crate::proto::services::peer_manager::peer_manager_client::PeerManagerClient;
        use crate::proto::services::peer_messaging_broker::peer_messaging_broker_client::PeerMessagingBrokerClient;

        #[derive(Debug, Clone)]
        pub struct CarlClient {
            pub broker: PeerMessagingBroker<tonic::transport::Channel>,
            pub cluster: ClusterManager<tonic::transport::Channel>,
            pub metadata: MetadataProvider<tonic::transport::Channel>,
            pub peers: PeersRegistrar<tonic::transport::Channel>,
        }

        impl CarlClient {

            pub fn create(host: impl Into<String>, port: u16, ca_cert_path: std::path::PathBuf, domain_name_override: Option<String>) -> Result<CarlClient, InitializationError> {

                let address = format!("https://{}:{}", host.into(), port);

                let tls_config = {
                    log::debug!("Using TLS CA certificate: {}", ca_cert_path.display());
                    let ca_cert = std::fs::read_to_string(&ca_cert_path)
                        .map_err(|cause| InitializationError::TlsConfiguration { message: format!("Failed to read CA certificate from path '{}'", ca_cert_path.display()), cause: cause.into() })?;

                    let mut config = tonic::transport::ClientTlsConfig::new()
                        .ca_certificate(tonic::transport::Certificate::from_pem(ca_cert));

                    if let Some(domain_name_override) = domain_name_override {
                        log::debug!("Using override for verified domain name of '{domain_name_override}'.");
                        config = config.domain_name(domain_name_override);
                    }
                    config
                };

                let endpoint = tonic::transport::Channel::from_shared(address.clone())
                    .map_err(|cause| InitializationError::InvalidUri {
                        uri: address.clone(),
                        cause,
                    })?
                    .tls_config(tls_config)
                    .map_err(|cause| InitializationError::TlsConfiguration { message: String::from("Failed to initialize secure channel with specified TLS configuration"), cause: cause.into() })?;

                log::debug!("Set up endpoint for connection to CARL at '{address}'.");
                let channel = endpoint.connect_lazy();

                Ok(CarlClient {
                    broker: PeerMessagingBroker::new(PeerMessagingBrokerClient::new(Clone::clone(&channel))),
                    cluster: ClusterManager::new(ClusterManagerClient::new(Clone::clone(&channel))),
                    metadata: MetadataProvider::new(MetadataProviderClient::new(Clone::clone(&channel))),
                    peers: PeersRegistrar::new(PeerManagerClient::new(Clone::clone(&channel))),
                })
            }
        }
    }
}

#[cfg(feature = "wasm-client")]
pub mod wasm {
    use crate::carl::broker::PeerMessagingBroker;
    use crate::carl::cluster::ClusterManager;
    use crate::carl::InitializationError;
    use crate::carl::metadata::MetadataProvider;
    use crate::carl::peer::PeersRegistrar;
    use crate::proto::services::cluster_manager::cluster_manager_client::ClusterManagerClient;
    use crate::proto::services::metadata_provider::metadata_provider_client::MetadataProviderClient;
    use crate::proto::services::peer_manager::peer_manager_client::PeerManagerClient;
    use crate::proto::services::peer_messaging_broker::peer_messaging_broker_client::PeerMessagingBrokerClient;

    #[derive(Debug, Clone)]
    pub struct CarlClient {
        pub broker: PeerMessagingBroker<tonic_web_wasm_client::Client>,
        pub cluster: ClusterManager<tonic_web_wasm_client::Client>,
        pub metadata: MetadataProvider<tonic_web_wasm_client::Client>,
        pub peers: PeersRegistrar<tonic_web_wasm_client::Client>,
    }

    impl CarlClient {

        pub fn create(url: url::Url) -> Result<CarlClient, InitializationError> {

            let scheme = url.scheme();
            let host = url.host_str().unwrap_or("localhost");
            let port = url.port().unwrap_or(8080_u16);

            let client = tonic_web_wasm_client::Client::new(format!("{}://{}:{}", scheme, host, port));

            Ok(CarlClient {
                broker: PeerMessagingBroker::new(PeerMessagingBrokerClient::new(Clone::clone(&client))),
                cluster: ClusterManager::new(ClusterManagerClient::new(Clone::clone(&client))),
                metadata: MetadataProvider::new(MetadataProviderClient::new(Clone::clone(&client))),
                peers: PeersRegistrar::new(PeerManagerClient::new(Clone::clone(&client))),
            })
        }
    }
}
