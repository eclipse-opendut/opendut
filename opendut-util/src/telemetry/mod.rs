pub mod opentelemetry_types;
pub mod logging;
mod traces;
pub mod metrics;

use std::fmt::Debug;
use std::fs::File;
use std::str::FromStr;
use std::sync::Arc;

use opentelemetry::global;
use opentelemetry::global::{GlobalLoggerProvider, logger_provider};
use opentelemetry::trace::TraceError;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::logs::Logger;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use tokio::sync::Mutex;
use tracing::error;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use opendut_auth::confidential::blocking::client::{AuthError, ConfClientArcMutex};
use opendut_auth::confidential::error::ConfidentialClientError;
use crate::telemetry::logging::{LoggingConfig, LoggingConfigError};
use crate::telemetry::metrics::{NamedMeterProvider, NamedMeterProviderKindCpu, NamedMeterProviderKindDefault, NamedMeterProviders};
use crate::telemetry::opentelemetry_types::{Opentelemetry, OpentelemetryConfigError};

pub const DEFAULT_METER_NAME: &str = "opendut_meter";

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
    #[error("Failed to get token from AuthenticationManager")]
    FailedToGetTokenFromAuthenticationManager { #[from] source: AuthError },
    #[error("Failed to create LoggingConfig: {cause}")]
    LoggingConfigError { #[from] cause: LoggingConfigError },
    #[error("Failed to create OpenTelemetryConfig: {cause}")]
    OpenTelemetryConfigError { #[from] cause: OpentelemetryConfigError },
    #[error("Failed to create ConfidentialClient: {cause}")]
    ConfidentialClientError { #[from] cause: ConfidentialClientError },
}

pub async fn initialize_with_config(
    logging_config: LoggingConfig,
    opentelemetry_config: Opentelemetry,
) -> Result<ShutdownHandle, Error> {

    global::set_text_map_propagator(TraceContextPropagator::new());
    
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(Directive::from_str("opendut=trace")?)
        .with_env_var("OPENDUT_LOG")
        .from_env()?;

    let stdout_logging_layer =
        if logging_config.logging_stdout {
            let stdout_logging_layer = tracing_subscriber::fmt::layer()
                .compact();
            Some(stdout_logging_layer)
        } else {
            None
        };

    let file_logging_layer =
        if let Some(log_file) = logging_config.file_logging {

            let log_file = File::options()
                .append(true)
                .create(true)
                .open(&log_file)
                .unwrap_or_else(|cause| panic!("Failed to open log file at '{}': {cause}", log_file.display()));

            Some(tracing_subscriber::fmt::layer()
                .with_writer(log_file))
        } else {
            None
        };

    let (tracer, logger, logger_layer, meter_providers) =
        if let Opentelemetry::Enabled {
            collector_endpoint,
            service_name, 
            service_instance_id, 
            metrics_interval_ms, 
            cpu_collection_interval_ms, 
            confidential_client, ..} = opentelemetry_config {

            let confidential_client = ConfClientArcMutex(Arc::new(Mutex::new(confidential_client)));

            let tracer = traces::init_tracer(confidential_client.clone(), &collector_endpoint, service_name.clone(), service_instance_id.clone()).expect("Failed to initialize tracer.");
            
            let logger = logging::init_logger(confidential_client.clone(), &collector_endpoint, service_name.clone(), service_instance_id.clone()).expect("Failed to initialize logs.");

            let logger_provider: GlobalLoggerProvider = logger_provider();
            let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

            let default_meter_provider = NamedMeterProvider {
                kind: NamedMeterProviderKindDefault,
                meter_provider: metrics::init_metrics(
                    confidential_client.clone(),
                    &collector_endpoint,
                    service_name.clone(),
                    service_instance_id.clone(),
                    metrics_interval_ms
                ).expect("Failed to initialize default metrics.")
            };

            let cpu_meter_provider = NamedMeterProvider {
                kind: NamedMeterProviderKindCpu,
                meter_provider: metrics::init_metrics(
                    confidential_client,
                    &collector_endpoint,
                    service_name,
                    service_instance_id,
                    cpu_collection_interval_ms
                ).expect("Failed to initialize CPU metrics.")
            };

            global::set_meter_provider(default_meter_provider.meter_provider.clone());
            let meter_providers: NamedMeterProviders = (default_meter_provider, cpu_meter_provider);

            (Some(tracer), Some(logger), Some(logger_layer), Some(meter_providers))
    } else {
        (None, None, None, None)
    };

    tracing_subscriber::registry()
        .with(stdout_logging_layer)
        .with(tracing_filter)
        .with(file_logging_layer)
        .with(tracer.map(|tracer| tracing_opentelemetry::layer().with_tracer(tracer)))
        .with(logger_layer)
        .try_init()?;

    Ok(ShutdownHandle { _logger: logger, meter_providers })
}

pub async fn initialize_with_defaults() -> Result<ShutdownHandle, Error> {
    initialize_with_config(LoggingConfig::default(), Opentelemetry::Disabled).await
}

#[must_use]
pub struct ShutdownHandle {
    pub(crate) _logger: Option<Logger>,
    pub meter_providers: Option<(
        NamedMeterProvider<NamedMeterProviderKindDefault>,
        NamedMeterProvider<NamedMeterProviderKindCpu>
    )>,
}
impl ShutdownHandle {
    pub fn shutdown(&mut self) {
        global::shutdown_tracer_provider();
        global::shutdown_logger_provider();
        //global::shutdown_meter_provider(); //TODO re-include when this appears in a release: https://github.com/open-telemetry/opentelemetry-rust/pull/1623
    }
}
impl Drop for ShutdownHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}
