use std::time::Duration;
use url::Url;
use opendut_auth::confidential::client::{ConfidentialClient, ConfidentialClientRef};
use opendut_auth::confidential::error::ConfidentialClientError;

#[derive(Default)]
pub enum Opentelemetry {
    Enabled {
        confidential_client: Option<ConfidentialClientRef>,
        collector_endpoint: Endpoint,
        service_name: String,
        service_metadata: ServiceMetadata,
        metrics_interval_ms: Duration,
        cpu_collection_interval_ms: Duration,
    },
    #[default]
    Disabled,
}

impl Opentelemetry {
    pub async fn load(config: &config::Config, service_metadata: ServiceMetadata) -> Result<Self, OpentelemetryConfigError> {
        let field = String::from("opentelemetry.enabled");
        let opentelemetry_enabled = config.get_bool("opentelemetry.enabled")
            .map_err(|cause| OpentelemetryConfigError::ValueParseError{
                field: field.clone(),
                cause: format!("{:?}", cause)
            })?;

        if opentelemetry_enabled {
            let collector_endpoint = {
                let field = String::from("opentelemetry.collector.endpoint");
                let url = config.get_string(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{:?}", cause)
                    })?;
                let url = Url::parse(&url)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field,
                        cause: format!("{:?}", cause)
                    })?;
                Endpoint { url }
            };
            let service_name = {
                let field = String::from("opentelemetry.service.name");
                config.get_string(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{:?}", cause)
                    })?
            };

            let metrics_interval_ms = {
                let field = String::from("opentelemetry.metrics.interval.ms");

                let interval_i64 = config.get_int(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{:?}", cause)
                    })?;

                let interval_u64 = u64::try_from(interval_i64)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{:?}", cause)
                    })?;

                Duration::from_millis(interval_u64)
            };

            let cpu_collection_interval_ms = {
                let field = String::from("opentelemetry.metrics.cpu.collection.interval.ms");

                let interval_i64 = config.get_int(&field)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{:?}", cause)
                    })?;

                let interval_u64 = u64::try_from(interval_i64)
                    .map_err(|cause| OpentelemetryConfigError::ValueParseError {
                        field: field.clone(),
                        cause: format!("{:?}", cause)
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

            Ok(Opentelemetry::Enabled {
                confidential_client,
                collector_endpoint,
                service_name,
                service_metadata,
                metrics_interval_ms,
                cpu_collection_interval_ms,
            })
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
