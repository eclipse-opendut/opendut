use std::fmt::Debug;
use std::path::PathBuf;
use opentelemetry::KeyValue;
use opentelemetry::logs::LogError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use opendut_auth::confidential::blocking::client::{ConfClientArcMutex, ConfidentialClientRef};
use crate::telemetry::opentelemetry_types::Endpoint;

#[derive(Default, Clone)]
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
    service_name: impl Into<String>,
    service_instance_id: impl Into<String>
) -> Result<opentelemetry_sdk::logs::LoggerProvider, LogError> {

    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(
            opentelemetry_sdk::logs::Config::default().with_resource(Resource::new(vec![
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.into()),
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                    service_instance_id.into()
                ),
            ])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_interceptor(telemetry_interceptor)
                .with_endpoint(Clone::clone(&endpoint.url)),
        )
        .install_batch(runtime::Tokio)
}

#[derive(Debug, thiserror::Error)]
pub enum LoggingConfigError {
    #[error("Failed to parse configuration from field: '{field}'")]
    ValueParseError {
        field: String,
    },
}
