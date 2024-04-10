use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use sysinfo::{Pid, System};

use opentelemetry::{global, KeyValue};
use opentelemetry::global::{GlobalLoggerProvider, logger_provider};
use opentelemetry::logs::LogError;
use opentelemetry::metrics::MetricsError;
use opentelemetry::trace::TraceError;
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::logs::Logger;
use opentelemetry_sdk::metrics::SdkMeterProvider;
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

    let stdout_logging_layer =
        if config.logging_stdout {
            let stdout_logging_layer = tracing_subscriber::fmt::layer()
                .compact();
            Some(stdout_logging_layer)
        } else {
            None
        };

    let file_logging_layer =
        if let Some(log_file) = config.file_logging {

            let log_file = File::create(&log_file)
                .unwrap_or_else(|cause| panic!("Failed to open log file at '{}': {cause}", log_file.display()));

            Some(tracing_subscriber::fmt::layer()
                .with_writer(log_file))
        } else {
            None
        };

    let (tracer, logger, logger_layer, meter_provider) = if let Some(endpoint) = config.opentelemetry_endpoint {
        let service_name: String = config.opentelemetry_service_name.clone()
            .unwrap_or_default();
        let service_instance_id = config.opentelemetry_service_instance_id.clone().unwrap_or_else(|| String::from("carl_instance"));
        let tracer = init_tracer(&endpoint, service_name, service_instance_id).expect("Failed to initialize tracer.");

        let service_name: String = config.opentelemetry_service_name.clone()
            .unwrap_or_default();
        let service_instance_id = config.opentelemetry_service_instance_id.clone().unwrap_or_else(|| String::from("carl_instance"));
        let logger = init_logger(&endpoint, service_name, service_instance_id).expect("Failed to initialize logs.");

        let logger_provider: GlobalLoggerProvider = logger_provider();
        let logger_layer = OpenTelemetryTracingBridge::new(&logger_provider);

        let service_name: String = config.opentelemetry_service_name
            .unwrap_or_default();
        let service_instance_id = config.opentelemetry_service_instance_id.unwrap_or_else(|| String::from("carl_instance"));
        let metrics_interval = config.opentelemetry_metrics_interval_ms.unwrap_or_default();
        let meter_provider = init_metrics(&endpoint, service_name, service_instance_id, metrics_interval).expect("Failed to initialize metrics.");

        (Some(tracer), Some(logger), Some(logger_layer), Some(meter_provider))
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

    Ok(ShutdownHandle { _logger: logger, _meter_provider: meter_provider })
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

fn init_logger(endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::logs::Logger, LogError> {
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
                .with_endpoint(Clone::clone(&endpoint.url)),
        )
        .install_batch(runtime::Tokio)
}

fn init_metrics(endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>, metrics_interval: Duration) -> Result<SdkMeterProvider, MetricsError> {
    opentelemetry_otlp::new_pipeline()
        .metrics(runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
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
pub fn initialize_metrics_collection() {
    let meter = global::meter("opendut_meter");
    
    let current_pid = std::process::id() as usize;
    let process_ram_used = meter.u64_observable_gauge("process_ram_used").init();
    let process_cpu_used = meter.f64_observable_gauge("process_cpu_used").init();
    let host_ram_used = meter.u64_observable_gauge("host_ram_used").init();
    
    meter.register_callback(&[process_ram_used.as_any(),process_cpu_used.as_any(),host_ram_used.as_any()], move |observer| {
        let mut sys = System::new_all();
        sys.refresh_processes();

        if let Some(process) = sys.process(Pid::from(current_pid)) {
            observer.observe_u64(&process_ram_used, process.memory(),&[]);
            observer.observe_f64(&process_cpu_used, process.cpu_usage() as f64,&[]);
            observer.observe_u64(&host_ram_used, sys.used_memory(),&[]);                
        }
    }).expect("could not register metrics collection callback");
}


#[derive(Default)]
pub struct LoggingConfig {
    pub file_logging: Option<PathBuf>,
    pub logging_stdout: bool,
    pub opentelemetry_endpoint: Option<Endpoint>,
    pub opentelemetry_service_name: Option<String>,
    pub opentelemetry_service_instance_id: Option<String>,
    pub opentelemetry_metrics_interval_ms: Option<Duration>,
}
impl LoggingConfig {
    pub fn load(config: &config::Config) -> Result<Self, LoggingConfigError> {
        let file_logging = None; //TODO load from config
        let logging_stdout = config.get_bool("opentelemetry.logging.stdout")?;

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
            let field = String::from("opentelemetry.service.name");
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
        let opentelemetry_metrics_interval_ms:Option<Duration> = if opentelemetry_enabled {
            let field = String::from("opentelemetry.metrics.interval.ms");
            let result =
                if let Ok(interval_i64) = config.get_int(&field) {
                    let interval_u64 = u64::try_from(interval_i64);
                    if let Ok(result_u64) = interval_u64 {
                        Duration::from_millis(result_u64)
                    } else {
                        Duration::from_millis(60000)
                    }
                } else {
                    Duration::from_millis(60000)
                };
            Some(result)
        } else {
            None
        };

        Ok(LoggingConfig {
            file_logging,
            logging_stdout,
            opentelemetry_endpoint,
            opentelemetry_service_name,
            opentelemetry_service_instance_id,
            opentelemetry_metrics_interval_ms
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
    _meter_provider: Option<SdkMeterProvider>,
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