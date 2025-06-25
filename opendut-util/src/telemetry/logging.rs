use crate::telemetry::opentelemetry_types::Endpoint;
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use opentelemetry_otlp::{ExporterBuildError, LogExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::logs::{SdkLoggerProvider};
use opentelemetry_sdk::Resource;
use std::fmt::Debug;
use std::path::PathBuf;
use serde::Deserialize;

#[derive(Default)]
pub struct LoggingConfig {
    pub pipe_logging: PipeLogging,
    pub file_logging: Option<PathBuf>,
}

#[derive(Default)]
pub enum PipeLogging {
    #[default]
    Disabled,
    Enabled { stream: PipeLoggingStream },
}

#[derive(Deserialize)]
#[serde(rename_all="lowercase")]
pub enum PipeLoggingStream { Stdout, Stderr }

impl LoggingConfig {
    pub fn load(config: &config::Config) -> Result<Self, LoggingConfigError> {

        let pipe_logging_enabled = {
            let field = String::from("logging.pipe.enabled");
            config.get_bool(&field)
                .map_err(|_cause| LoggingConfigError::ValueParseError {
                    field,
                })?
        };

        let pipe_logging =
            if pipe_logging_enabled {
                let stream = {
                    let field = String::from("logging.pipe.stream");
                    config.get::<PipeLoggingStream>(&field)
                        .map_err(|_cause| LoggingConfigError::ValueParseError {
                            field,
                        })?
                };

                PipeLogging::Enabled { stream }
            } else {
                PipeLogging::Disabled
            };

        Ok(LoggingConfig {
            pipe_logging,
            file_logging: None,
        })
    }
}

pub fn init_logger_provider(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_metadata_resource: Resource,
) -> Result<SdkLoggerProvider, ExporterBuildError> {

    let exporter = LogExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let provider = SdkLoggerProvider::builder()
        .with_resource(service_metadata_resource)
        .with_batch_exporter(exporter)
        .build();

    Ok(provider)
}

#[derive(Debug, thiserror::Error)]
pub enum LoggingConfigError {
    #[error("Failed to parse configuration from field: '{field}'")]
    ValueParseError {
        field: String,
    },
}
