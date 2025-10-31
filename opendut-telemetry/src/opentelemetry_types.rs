use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::error::ConfidentialClientError;
use std::time::Duration;
use pem::Pem;
use tonic::transport::{Certificate, ClientTlsConfig};
use url::Url;
use opendut_util_core::pem::PemFromConfig;

pub struct OpentelemetryConfig {
    pub(crate) confidential_client: Option<ConfidentialClientRef>,
    pub(crate) collector_endpoint: Endpoint,
    pub(crate) service_name: String,
    pub(crate) service_metadata: ServiceMetadata,
    pub(crate) metrics_interval_ms: Duration,
    pub(crate) cpu_collection_interval_ms: Duration,
    pub(crate) client_tls_config: ClientTlsConfig,
}

#[derive(Default)]
pub enum Opentelemetry {
    Enabled(Box<OpentelemetryConfig>),
    #[default]
    Disabled,
}

impl Opentelemetry {
    pub async fn load(config: &config::Config, service_metadata: ServiceMetadata) -> Result<Self, OpentelemetryConfigError> {
        let field = String::from("opentelemetry.enabled");
        let opentelemetry_enabled = config.get_bool("opentelemetry.enabled")
            .map_err(|cause| OpentelemetryConfigError::ValueParseError{
                field: field.clone(),
                cause: format!("{cause:?}")
            })?;

        if opentelemetry_enabled {
            let collector_endpoint = {
                let field = String::from("opentelemetry.collector.endpoint");
                let url = config.get_string(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{cause:?}")
                    })?;
                let url = Url::parse(&url)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field,
                        cause: format!("{cause:?}")
                    })?;
                Endpoint { url }
            };
            let service_name = {
                let field = String::from("opentelemetry.service.name");
                config.get_string(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{cause:?}")
                    })?
            };

            let metrics_interval_ms = {
                let field = String::from("opentelemetry.metrics.interval.ms");

                let interval_i64 = config.get_int(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{cause:?}")
                    })?;

                let interval_u64 = u64::try_from(interval_i64)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{cause:?}")
                    })?;

                Duration::from_millis(interval_u64)
            };

            let cpu_collection_interval_ms = {
                let field = String::from("opentelemetry.metrics.cpu.collection.interval.ms");

                let interval_i64 = config.get_int(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{cause:?}")
                    })?;

                let interval_u64 = u64::try_from(interval_i64)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{cause:?}")
                    })?;
                let interval = Duration::from_millis(interval_u64);

                if interval < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
                    return Err(OpentelemetryConfigError::InvalidValueError {
                        field,
                        message: format!(
                            "Provided configuration value needs to be higher than the minimum CPU update interval of {} ms.",
                            sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis()
                        )
                    });
                }
                interval
            };
            let confidential_client = ConfidentialClient::from_settings(config).await
                .map_err(|cause| OpentelemetryConfigError::ConfidentialClientError{
                    message: String::from("Could not create AuthenticationManager"),
                    cause
                })?;

            let opendut_ca = Pem::read_from_config_with_env_fallback("network.tls.ca.content",
                "opentelemetry.client.ca",
                "network.tls.ca",
                config
            ).map_err(|cause| OpentelemetryConfigError::ValueParseError{
                    field: String::from("opentelemetry.client.ca"),
                    cause: format!("{cause:?}")
                })?;

            let mut client_tls_config = ClientTlsConfig::new()
                .with_enabled_roots();
            if let Some(opendut_ca) = opendut_ca {
                let certificate = Certificate::from_pem(opendut_ca.contents());
                client_tls_config = client_tls_config.ca_certificate(certificate);
            } else {
                client_tls_config = client_tls_config.with_native_roots();
            }

            Ok(Opentelemetry::Enabled(Box::new(OpentelemetryConfig {
                confidential_client,
                collector_endpoint,
                service_name,
                service_metadata,
                metrics_interval_ms,
                cpu_collection_interval_ms,
                client_tls_config,
            })))
        } else {
            Ok(Opentelemetry::Disabled)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum OpentelemetryConfigError {
    #[error("Failed to parse configuration from field: '{field}'. Cause: {cause}")]
    ValueParseError {
        field: String,
        cause: String
    },
    #[error("'{message}': '{field}'")]
    InvalidValueError {
        field: String,
        message: String,
    },
    #[error("'{message}': '{cause}'")]
    ConfidentialClientError {
        message: String,
        cause: ConfidentialClientError,
    }
}

#[derive(Debug)]
pub struct ServiceMetadata {
    pub instance_id: String,
    pub version: String,
}

#[derive(Clone)]
pub struct Endpoint {
    pub url: Url,
}
