use std::fmt::Debug;
use std::path::PathBuf;
use opentelemetry::KeyValue;
use opentelemetry::logs::LogError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use tonic::metadata::KeyAndValueRef;
use tonic::Request;
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

pub fn init_logger(telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>, endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
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

pub trait NonDisclosingRequestExtension {
    fn debug_output(&self) -> String;
}
impl <T:Debug> NonDisclosingRequestExtension for Request<T>
    where
        Request<T>: std::fmt::Debug {
    fn debug_output(&self) -> String {
        let metadata = self.metadata().clone();
        let message = self.get_ref();

        let mut headers = String::new();

        for key_value in metadata.iter().filter_map(|key| match key {
            KeyAndValueRef::Ascii(key, value) => {
                if !key.as_str().to_lowercase().contains("authorization") {
                    Some((key, value))
                } else {
                    None
                }
            }
            KeyAndValueRef::Binary(_, _) => None,
        }) {
            let (key, value) = key_value;
            let key_str = key.to_string();
            let value_str = value.to_str().unwrap_or_default().to_string();
            headers.push_str(&format!("{}: {}, ", key_str, value_str));
        }
        if headers.ends_with(", ") {
            headers.pop();
            headers.pop();
        }

        format!("Request {{ headers: {{ {} }}, message: {:?} }}", headers, message)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoggingConfigError {
    #[error("Failed to parse configuration from field: '{field}'")]
    ValueParseError {
        field: String,
    },
}