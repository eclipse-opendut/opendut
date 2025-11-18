use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::error::ConfidentialClientError;
use std::time::Duration;
use pem::Pem;
use tonic::transport::{Certificate, ClientTlsConfig};
use url::Url;
use opendut_util_core::pem::PemFromConfig;
use std::fmt::Debug;

pub struct OpentelemetryConfig {
    pub(crate) confidential_client: Option<ConfidentialClientRef>,
    pub(crate) collector_endpoint: Endpoint,
    pub(crate) service_name: String,
    pub(crate) service_metadata: ServiceMetadata,
    pub(crate) metrics_interval_ms: Duration,
    pub(crate) cpu_collection_interval_ms: Duration,
    pub(crate) client_tls_config: ClientTlsConfig,
}

impl PartialEq for OpentelemetryConfig {
    fn eq(&self, other: &Self) -> bool {
        self.collector_endpoint == other.collector_endpoint &&
        self.service_name == other.service_name &&
        self.service_metadata == other.service_metadata &&
        self.metrics_interval_ms == other.metrics_interval_ms &&
        self.cpu_collection_interval_ms == other.cpu_collection_interval_ms
    }
}

impl Debug for OpentelemetryConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OpentelemetryConfig")
            .field("collector_endpoint", &self.collector_endpoint)
            .field("service_name", &self.service_name)
            .field("service_metadata", &self.service_metadata)
            .field("metrics_interval_ms", &self.metrics_interval_ms)
            .field("cpu_collection_interval_ms", &self.cpu_collection_interval_ms)
            .finish()
    }
}

#[derive(Default, Debug, PartialEq)]
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
                let url = Url::parse_without_quotes(&url)
                    .map_err(|cause| OpentelemetryConfigError::InvalidValueError {
                        field,
                        message: format!("Failed to parse url from given string: '{url}'. Error: {cause:?}")
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

pub trait UrlWithoutQuotes {
    fn parse_without_quotes(url: &str) -> Result<Url, url::ParseError>;
}

impl UrlWithoutQuotes for Url {
    fn parse_without_quotes(url: &str) -> Result<Url, url::ParseError> {
        let url_str = url
            .trim_matches('"')
            .trim_matches('\'');
        Url::parse(url_str)
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

#[derive(Debug, PartialEq)]
pub struct ServiceMetadata {
    pub instance_id: String,
    pub version: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Endpoint {
    pub url: Url,
}


#[cfg(test)]
mod tests {
    use std::time::Duration;
    use url::Url;
    use crate::opentelemetry_types::{Endpoint, Opentelemetry, OpentelemetryConfig, ServiceMetadata, UrlWithoutQuotes};

    #[test]
    fn should_parse_url_without_quotes() -> anyhow::Result<()> {
        let url = "http://otel-collector:4317";

        let url = Url::parse_without_quotes(url)?;
        assert!(url.as_str().starts_with("http://otel-collector:4317"));

        Ok(())
    }

    #[test]
    fn should_parse_url_with_quotes() -> anyhow::Result<()> {
        let url = "'http://otel-collector:4317'";

        let url = Url::parse_without_quotes(url)?;
        assert!(url.as_str().starts_with("http://otel-collector:4317"));

        Ok(())
    }

    #[tokio::test]
    async fn test_open_telemetry_config_load() -> anyhow::Result<()> {
        let otel_collector = "http://otel-collector:4317";
        let instance_name = "instance-1";
        let test_service_name = "test-service";
        let test_service_version = "v1.0.0";
        let url = Url::parse(otel_collector).expect("Could not parse ote.l est URL");

        let config = config::Config::builder()
            .set_override("opentelemetry.enabled", "true")?
            .set_override("opentelemetry.service.name", test_service_name)?
            .set_override("opentelemetry.collector.endpoint", otel_collector)?
            .set_override("opentelemetry.metrics.interval.ms", "1000")?
            .set_override("opentelemetry.metrics.cpu.collection.interval.ms", "1000")?
            .set_override("network.oidc.enabled", "false")?
            .build()?;

        let otel = Opentelemetry::load(&config, ServiceMetadata {
            instance_id: String::from(instance_name),
            version: String::from(test_service_version),
        }).await?;
        let expected = Opentelemetry::Enabled(Box::new(OpentelemetryConfig {
            confidential_client: None,
            collector_endpoint: Endpoint { url },
            service_name: test_service_name.to_string(),
            service_metadata: ServiceMetadata { instance_id: instance_name.to_string(), version: test_service_version.to_string() },
            metrics_interval_ms: Duration::from_secs(1),
            cpu_collection_interval_ms: Duration::from_secs(1),
            client_tls_config: Default::default(),
        }));

        assert_eq!(otel, expected);

        Ok(())

    }

}