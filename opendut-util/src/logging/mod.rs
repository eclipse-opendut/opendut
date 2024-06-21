use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;

use opentelemetry::{global, KeyValue};
use opentelemetry::global::{GlobalLoggerProvider, logger_provider};
use opentelemetry::logs::LogError;
use opentelemetry::metrics::{MeterProvider, MetricsError};
use opentelemetry::trace::TraceError;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::logs::Logger;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use simple_moving_average::{SMA, SumTreeSMA};
use sysinfo::{Pid, System};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tonic::Request;
use tonic::metadata::KeyAndValueRef;
use tracing::error;
use tracing_subscriber::filter::Directive;
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use url::Url;
use opendut_auth::confidential::blocking::client::{AuthError, ConfidentialClientRef, ConfClientArcMutex};

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
}

pub async fn initialize_with_config(
    config: LoggingConfig,
    file_logging: Option<PathBuf>,
    confidential_client: Option<ConfidentialClientRef>,
) -> Result<ShutdownHandle, Error> {

    global::set_text_map_propagator(TraceContextPropagator::new());
    
    let tracing_filter = EnvFilter::builder()
        .with_default_directive(Directive::from_str("opendut=trace")?)
        .with_env_var("OPENDUT_LOG")
        .from_env()?;

    let stdout_logging_layer =
        if config.logging_stdout {
            let stdout_logging_layer = tracing_subscriber::fmt::layer()
                .compact();
            Some(stdout_logging_layer)
        } else {
            None
        };

    let file_logging_layer =
        if let Some(log_file) = file_logging {

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
        if let OpenTelemetryConfig::Enabled { endpoint, service_name, service_instance_id, metrics_interval_ms, cpu_collection_interval_ms, ..} = config.opentelemetry {

            let confidential_client = ConfClientArcMutex(Arc::new(Mutex::new(confidential_client)));

            let tracer = init_tracer(confidential_client.clone(), &endpoint, service_name.clone(), service_instance_id.clone()).expect("Failed to initialize tracer.");
            
            let logger = init_logger(confidential_client.clone(), &endpoint, service_name.clone(), service_instance_id.clone()).expect("Failed to initialize logs.");

            let logger_provider: GlobalLoggerProvider = logger_provider();
            let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

            let default_meter_provider = NamedMeterProvider {
                kind: NamedMeterProviderKindDefault,
                meter_provider: init_metrics(
                    confidential_client.clone(),
                    &endpoint,
                    service_name.clone(),
                    service_instance_id.clone(),
                    metrics_interval_ms
                ).expect("Failed to initialize default metrics.")
            };

            let cpu_meter_provider = NamedMeterProvider {
                kind: NamedMeterProviderKindCpu,
                meter_provider: init_metrics(
                    confidential_client,
                    &endpoint,
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

fn init_tracer(telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>, endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_interceptor(telemetry_interceptor)
                .with_endpoint(Clone::clone(&endpoint.url)),
        )
        .with_trace_config(
            opentelemetry_sdk::trace::config().with_resource(Resource::new(vec![
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.into()),
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                    service_instance_id.into()
                )
            ])),
        )
        .install_batch(runtime::Tokio)
}

fn init_logger(telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>, endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
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

fn init_metrics(telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>, endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>, metrics_interval: Duration) -> Result<SdkMeterProvider, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_interceptor(telemetry_interceptor)
                .with_endpoint(Clone::clone(&endpoint.url))
        )
        .with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                service_name.into()
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                service_instance_id.into()
            ),
        ]))
        .with_period(metrics_interval)
        .build()
}
pub fn initialize_metrics_collection(
    cpu_collection_interval_ms: Duration,
    meter_providers: &NamedMeterProviders,
) {
    let (default_meter_provider, cpu_meter_provider) = meter_providers;
    let default_meter = default_meter_provider.meter_provider.meter(DEFAULT_METER_NAME);

    let process_ram_used = default_meter.u64_observable_gauge("process_ram_used").init();
    let host_ram_used = default_meter.u64_observable_gauge("host_ram_used").init();

    const WINDOW_SIZE: usize = 5;
    
    let moving_average = SumTreeSMA::<f64, f64, WINDOW_SIZE>::new();
    let mutex = Arc::new(Mutex::new(moving_average));
    let mutex_cloned = Arc::clone(&mutex);

    tokio::spawn(async move {
        let current_pid = std::process::id() as usize;
        let mut sys = System::new_all();
        sys.refresh_processes();
        loop {
            sleep(cpu_collection_interval_ms).await;
            sys.refresh_processes();
            if let Some(process) = sys.process(Pid::from(current_pid)) {
                let result = process.cpu_usage();
                mutex_cloned.lock().await.add_sample(result as f64);
            }
        }
    });

    let cpu_meter = cpu_meter_provider.meter_provider.meter(DEFAULT_METER_NAME);
    let process_cpu_used = cpu_meter.f64_observable_gauge("process_cpu_used").init();
    cpu_meter.register_callback(&[process_cpu_used.as_any()], move |observer| {
        let average_cpu_usage = mutex.try_lock().unwrap().get_average();
        observer.observe_f64(&process_cpu_used, average_cpu_usage,&[]);
    }).expect("Could not register metrics collection callback");

    let current_pid = std::process::id() as usize;
    default_meter.register_callback(&[process_ram_used.as_any(), host_ram_used.as_any()], move |observer| {
        let mut sys = System::new_all();
        sys.refresh_processes();

        if let Some(process) = sys.process(Pid::from(current_pid)) {
            observer.observe_u64(&process_ram_used, process.memory(),&[]);
            observer.observe_u64(&host_ram_used, sys.used_memory(),&[]);
        }
    }).expect("Could not register metrics collection callback");

}

pub async fn initialize_with_defaults() -> Result<ShutdownHandle, Error> {
    initialize_with_config(LoggingConfig::default(), None, None).await
}

#[derive(Default, Clone)]
pub struct LoggingConfig {
    pub logging_stdout: bool,
    pub opentelemetry: OpenTelemetryConfig,
}
#[derive(Default, Clone)]
pub enum OpenTelemetryConfig {
    Enabled {
        endpoint: Endpoint,
        service_name: String,
        service_instance_id: String,
        metrics_interval_ms: Duration,
        cpu_collection_interval_ms: Duration,
    },
    #[default]
    Disabled,
}

impl LoggingConfig {
    pub fn load(config: &config::Config, service_instance_id: String) -> Result<Self, LoggingConfigError> {
        let logging_stdout = config.get_bool("opentelemetry.logging.stdout")?;

        let opentelemetry_enabled = config.get_bool("opentelemetry.enabled")?;

        let opentelemetry = if opentelemetry_enabled {
            let endpoint = {
                let field = String::from("opentelemetry.endpoint");
                let url = config.get_string(&field)?;
                let url = Url::parse(&url)
                    .map_err(|cause| LoggingConfigError::InvalidFieldValue {
                        field,
                        message: cause.to_string()
                    })?;
                Endpoint { url }
            };
            let service_name = {
                let field = String::from("opentelemetry.service.name");
                config.get_string(&field)
                    .map_err(|_cause| LoggingConfigError::InvalidFieldValue { field: field.clone(), message: String::from("Failed to parse configuration from field") })?
            };

            let metrics_interval_ms =  {
                let field = String::from("opentelemetry.metrics.interval.ms");

                let interval_i64 = config.get_int(&field)
                    .map_err(|_cause| LoggingConfigError::InvalidFieldValue { field: field.clone(), message: String::from("Failed to parse configuration from field") })?;

                let interval_u64 = u64::try_from(interval_i64)
                    .map_err(|_cause| LoggingConfigError::InvalidFieldValue { field: field.clone(), message: String::from("Failed to convert to u64.") })?;

                Duration::from_millis(interval_u64)
            };

            let cpu_collection_interval_ms = {
                let field = String::from("opentelemetry.cpu.collection.interval.ms");

                let interval_i64 = config.get_int(&field)
                    .map_err(|_cause| LoggingConfigError::InvalidFieldValue { field: field.clone(), message: String::from("Failed to parse configuration from field.") })?;

                let interval_u64 = u64::try_from(interval_i64)
                    .map_err(|_cause| LoggingConfigError::InvalidFieldValue { field: field.clone(), message: String::from("Failed to convert to u64.") })?;
                let interval = Duration::from_millis(interval_u64);

                if interval < sysinfo::MINIMUM_CPU_UPDATE_INTERVAL {
                    return Err(LoggingConfigError::InvalidFieldValue {
                        field,
                        message: format!("Provided configuration value needs to be higher than the minimum CPU update interval of {} ms.", sysinfo::MINIMUM_CPU_UPDATE_INTERVAL.as_millis())
                    });
                }
                interval
            };

            OpenTelemetryConfig::Enabled {
                endpoint,
                service_name,
                service_instance_id,
                metrics_interval_ms,
                cpu_collection_interval_ms
            }
        } else {
            OpenTelemetryConfig::Disabled
        };

        Ok(LoggingConfig {
            logging_stdout,
            opentelemetry,
        })
    }
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

pub struct NamedMeterProvider<Kind: NamedMeterProviderKind> {
    pub kind: Kind,
    pub meter_provider: SdkMeterProvider,
}
pub type NamedMeterProviders = (NamedMeterProvider<NamedMeterProviderKindDefault>, NamedMeterProvider<NamedMeterProviderKindCpu>);

pub trait NamedMeterProviderKind {}
pub struct NamedMeterProviderKindDefault;
impl NamedMeterProviderKind for NamedMeterProviderKindDefault {}
pub struct NamedMeterProviderKindCpu;
impl NamedMeterProviderKind for NamedMeterProviderKindCpu {}

#[must_use]
pub struct ShutdownHandle {
    _logger: Option<Logger>,
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
