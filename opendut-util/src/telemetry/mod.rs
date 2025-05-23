pub mod opentelemetry_types;
pub mod logging;
mod traces;
pub mod metrics;

use std::fmt::Debug;
use std::fs::File;
use std::str::FromStr;
use std::sync::Arc;

use opentelemetry::{global, KeyValue};
use opentelemetry::trace::{TracerProvider};
use opentelemetry_sdk::trace::{SdkTracerProvider, TraceError};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::Resource;
use tokio::sync::Mutex;
use tracing::level_filters::LevelFilter;
use tracing::{debug, error, trace};
use tracing_subscriber::filter::Directive;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use opendut_auth::confidential::client::{AuthError, ConfClientArcMutex};
use opendut_auth::confidential::error::ConfidentialClientError;
use crate::telemetry::logging::{LoggingConfig, LoggingConfigError};
use crate::telemetry::metrics::{NamedMeterProvider, NamedMeterProviderKindCpu, NamedMeterProviderKindDefault, NamedMeterProviders};
use crate::telemetry::opentelemetry_types::{Opentelemetry, OpentelemetryConfigError};

pub const DEFAULT_METER_NAME: &str = "opendut_meter";
pub const DEFAULT_TRACER_NAME: &str = "opendut_tracer";

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

    let tracing_subscriber = tracing_subscriber::registry()
        .with(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .with_env_var("OPENDUT_LOG")
                .from_env()?
                .add_directive(Directive::from_str("opendut=trace")?)
        ).with(
            logging_config.logging_stdout
                .then_some(tracing_subscriber::fmt::layer())
        ).with(
            logging_config.file_logging
                .map(|log_file| {
                    let log_file = File::options()
                        .append(true)
                        .create(true)
                        .open(&log_file)
                        .unwrap_or_else(|cause| panic!("Failed to open log file at '{}': {cause}", log_file.display()));

                    tracing_subscriber::fmt::layer()
                        .with_writer(log_file)
                })
        );

    let (meter_providers, tracer_provider) =
        if let Opentelemetry::Enabled {
            collector_endpoint,
            service_name,
            service_metadata,
            metrics_interval_ms,
            cpu_collection_interval_ms,
            confidential_client,
        } = opentelemetry_config {
            let confidential_client = ConfClientArcMutex(Arc::new(Mutex::new(confidential_client)));

            let service_metadata_resource = Resource::builder()
                .with_service_name(service_name.to_owned())
                .with_attributes(vec![
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                        service_metadata.instance_id.to_owned()
                    ),
                    KeyValue::new(
                        opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                        service_metadata.version.to_owned()
                    ),
                ]).build();

            let tracer_provider = traces::init_tracer(
                confidential_client.clone(),
                &collector_endpoint,
                service_metadata_resource.clone(),
            ).expect("Failed to initialize tracer.");

            let tracer = tracer_provider.tracer(DEFAULT_TRACER_NAME);
            let tracing_layer = tracing_opentelemetry::layer()
                .with_tracer(tracer);

            let logger_provider = logging::init_logger_provider(
                confidential_client.clone(),
                &collector_endpoint,
                service_metadata_resource.clone()
            ).expect("Failed to initialize logs.");

            let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

            let default_meter_provider = NamedMeterProvider {
                kind: NamedMeterProviderKindDefault,
                meter_provider: metrics::init_metrics(
                    confidential_client.clone(),
                    &collector_endpoint,
                    service_metadata_resource.clone(),
                    metrics_interval_ms
                ).expect("Failed to initialize default metrics.")
            };

            let cpu_meter_provider = NamedMeterProvider {
                kind: NamedMeterProviderKindCpu,
                meter_provider: metrics::init_metrics(
                    confidential_client,
                    &collector_endpoint,
                    service_metadata_resource.clone(),
                    cpu_collection_interval_ms
                ).expect("Failed to initialize CPU metrics.")
            };

            global::set_meter_provider(default_meter_provider.meter_provider.clone());
            let meter_providers: NamedMeterProviders = (default_meter_provider, cpu_meter_provider);

            tracing_subscriber
                .with(tracing_layer)
                .with(logger_layer)
                .try_init()?;

            trace!("Telemetry stack initialized with OpenTelemetry, using configuration:
endpoint:            {endpoint}
service_name:        {service_name}
service_metadata: {service_metadata:?}",
                endpoint=collector_endpoint.url
            );

            metrics::initialize_os_metrics_collection(cpu_collection_interval_ms, &meter_providers);

            (Some(meter_providers), Some(tracer_provider))
        } else {
            tracing_subscriber
                .try_init()?;

            trace!("Telemetry stack initialized without OpenTelemetry.");

            (None, None)
        };

    Ok(ShutdownHandle { meter_providers, tracer_provider })
}


#[must_use]
pub struct ShutdownHandle {
    pub meter_providers: Option<(
        NamedMeterProvider<NamedMeterProviderKindDefault>,
        NamedMeterProvider<NamedMeterProviderKindCpu>
    )>,
    tracer_provider: Option<SdkTracerProvider>,
}
impl ShutdownHandle {
    pub fn shutdown(&mut self) {
        debug!("Shutting down telemetry stack.");
        match self.tracer_provider.as_ref() {
            None => {}
            Some(tracer_provider) => {
                let _result = tracer_provider.shutdown()
                    .inspect_err(|err| error!("Failed to shut down telemetry stack: {}", err));
            }
        }
    }
}
impl Drop for ShutdownHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}
