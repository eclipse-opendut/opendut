use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;

use opentelemetry::{global, KeyValue};
use opentelemetry::global::{GlobalLoggerProvider, logger_provider};
use opentelemetry::logs::LogError;
use opentelemetry::trace::TraceError;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::logs::Logger;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Unable to initialize tracing: {source}")]
    TracingFilterFromEnv { #[from] source: tracing_subscriber::filter::FromEnvError },
    #[error("Unable to initialize tracing: {source}")]
    TracingFilterParse { #[from] source: tracing_subscriber::filter::ParseError },
    #[error("Unable to set initialize tracing: {source}")]
    TracingInit { #[from] source: tracing_subscriber::util::TryInitError },
    #[error("Unable to create the opentelemetry tracer: {source}")]
    Tracer { #[from] source: TraceError },
    #[error("No endpoint configuration provided.")]
    EndpointConfigurationMissing,
}

pub fn initialize() -> Result<ShutdownHandle, Error> {
    initialize_with_config(LoggingConfig::default())
}

pub fn initialize_with_config(config: LoggingConfig) -> Result<ShutdownHandle, Error> {

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracing_filter = EnvFilter::builder()
        .with_default_directive(Directive::from_str("opendut=trace")?)
        .with_env_var("OPENDUT_LOG")
        .from_env()?;

    let stdout_logging_layer = tracing_subscriber::fmt::layer()
        .compact();

    let file_logging_layer =
        if let Some(log_file) = config.file_logging {

            let log_file = File::create(&log_file)
                .unwrap_or_else(|cause| panic!("Failed to open log file at '{}': {cause}", log_file.display()));

            Some(tracing_subscriber::fmt::layer()
                .with_writer(log_file))
        } else {
            None
        };

    let (tracer, logger, logger_layer) = if let Some(endpoint) = config.opentelemetry_endpoint {
        let service_name: String = config.opentelemetry_service_name.clone()
            .unwrap_or_default();
        let service_instance_id = config.opentelemetry_service_instance_id.clone().unwrap_or_else(|| String::from("carl_instance"));
        let tracer = init_tracer(&endpoint, service_name, service_instance_id).expect("Failed to initialize tracer.");

        let service_name: String = config.opentelemetry_service_name
            .unwrap_or_default();
        let service_instance_id = config.opentelemetry_service_instance_id.unwrap_or_else(|| String::from("carl_instance"));
        let logger = init_logger(&endpoint, service_name, service_instance_id).expect("Failed to initialize logs.");

        let logger_provider: GlobalLoggerProvider = logger_provider();
        let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);
        (Some(tracer), Some(logger), Some(logger_layer))
    } else {
        (None, None, None)
    };

    tracing_subscriber::registry()
        .with(stdout_logging_layer)
        .with(tracing_filter)
        .with(file_logging_layer)
        .with(tracer.map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer)))
        .with(logger_layer)
        .try_init()?;

    Ok(ShutdownHandle { _logger: logger })
}

fn init_tracer(endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(Clone::clone(&endpoint.url)),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.into()),
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                    service_instance_id.into())
            ])),
        )
        .install_batch(runtime::Tokio)
}

fn init_logger(endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
    opentelemetry_otlp::new_pipeline()
        .logging()
        .with_log_config(
            opentelemetry_sdk::logs::Config::default().with_resource(Resource::new(vec![KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.into()),
                 KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                    service_instance_id.into())
            ])),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(Clone::clone(&endpoint.url)),
        )
        .install_batch(runtime::Tokio)
}

#[derive(Default)]
pub struct LoggingConfig {
    pub file_logging: Option<PathBuf>,
    pub opentelemetry_endpoint: Option<Endpoint>,
    pub opentelemetry_service_name: Option<String>,
    pub opentelemetry_service_instance_id: Option<String>,
}
impl LoggingConfig {
    pub fn load(config: &config::Config) -> Result<Self, LoggingConfigError> {
        let file_logging = None; //TODO load from config

        let opentelemetry_enabled = config.get_bool("opentelemetry.enabled")?;
        let opentelemetry_endpoint = if opentelemetry_enabled {
            let field = String::from("opentelemetry.endpoint");
            let url = config.get_string(&field)?;
            let url = Url::parse(&url)
                .map_err(|cause| LoggingConfigError::InvalidFieldValue {
                    field,
                    message: cause.to_string()
                })?;
            Some(Endpoint { url })
        } else {
            None
        };
        let opentelemetry_service_name = if opentelemetry_enabled {
            let field = String::from("opentelemetry.service_name");
            let service = config.get_string(&field)?;
            Some(service)
        } else {
            None
        };

        let opentelemetry_service_instance_id: Option<String> = if opentelemetry_enabled {
            let field = String::from("peer.id");
            config.get_string(&field).ok()
        } else {
            None
        };

        Ok(LoggingConfig {
            file_logging,
            opentelemetry_endpoint,
            opentelemetry_service_name,
            opentelemetry_service_instance_id,
        })
    }
}
#[derive(Debug, thiserror::Error)]
pub enum LoggingConfigError {
    #[error("Error while loading config: {source}")]
    Load { #[from] source: config::ConfigError },
    #[error("Invalid field '{field}': {message}")]
    InvalidFieldValue {
        field: String,
        message: String,
    },
}
#[derive(Clone)]
pub struct Endpoint {
    pub url: Url,
}
#[must_use]
pub struct ShutdownHandle {
    _logger: Option<Logger>,
}
impl ShutdownHandle {
    pub fn shutdown(&mut self) {
        global::shutdown_tracer_provider();
        global::shutdown_logger_provider();
    }
}
impl Drop for ShutdownHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}