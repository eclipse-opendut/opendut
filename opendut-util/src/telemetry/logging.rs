use std::fmt::Debug;
use std::path::PathBuf;
use opentelemetry::KeyValue;
use opentelemetry_sdk::logs::{LogError, LoggerProvider};
use opentelemetry_otlp::{LogExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{Resource, runtime};
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use crate::telemetry::opentelemetry_types::Endpoint;

#[derive(Default)]
pub struct LoggingConfig {
    pub logging_stdout: bool,
    pub file_logging: Option<PathBuf>,
}

impl LoggingConfig {
    pub fn load(config: &config::Config) -> Result<Self, LoggingConfigError> {
        let field = String::from("logging.stdout");
        let logging_stdout = config.get_bool(&field)
            .map_err(|_cause| LoggingConfigError::ValueParseError {
                field: field.clone(),
            })?;

        Ok(LoggingConfig {
            logging_stdout,
            file_logging: None,
        })
    }
}

pub fn init_logger_provider(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_name: &str,
    service_instance_id: &str,
) -> Result<LoggerProvider, LogError> {

    let exporter = LogExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let provider = LoggerProvider::builder()
        .with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name.to_owned()
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                service_instance_id.to_owned()
            ),
        ]))
        .with_batch_exporter(exporter, runtime::Tokio)
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
