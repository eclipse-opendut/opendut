use crate::telemetry::opentelemetry_types::Endpoint;
use crate::telemetry::DEFAULT_METER_NAME;
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use opentelemetry::metrics::MeterProvider;
use opentelemetry_otlp::{ExporterBuildError, MetricExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::metrics::PeriodicReader;
use opentelemetry_sdk::metrics::{SdkMeterProvider};
use opentelemetry_sdk::{Resource};
use simple_moving_average::{SumTreeSMA, SMA};
use std::sync::Arc;
use std::time::Duration;
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::sync::Mutex;
use tokio::time::sleep;
use tracing::trace;


pub(super) fn init_metrics(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_metadata_resource: Resource,
    metrics_interval: Duration
) -> Result<SdkMeterProvider, ExporterBuildError> {

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let reader = PeriodicReader::builder(exporter)
        .with_interval(metrics_interval)
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(service_metadata_resource)
        .build();

    Ok(provider)
}

pub(super) fn initialize_os_metrics_collection(
    cpu_collection_interval_ms: Duration,
    meter_providers: &NamedMeterProviders,
) {
    let (default_meter_provider, cpu_meter_provider) = meter_providers;
    let default_meter = default_meter_provider.meter_provider.meter(DEFAULT_METER_NAME);

    let current_pid = Pid::from_u32(std::process::id());
    default_meter.u64_observable_gauge("process_ram_used")
        .with_callback(move |observer| {
            let mut system = System::new_all();
            system.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), false);

            if let Some(process) = system.process(current_pid) {
                observer.observe(process.memory(), &[]);
            }
        })
        .build();

    default_meter.u64_observable_gauge("host_ram_used")
        .with_callback(|observer| {
            let system = System::new_all();
            observer.observe(system.used_memory(), &[]);
        })
        .build();

    const WINDOW_SIZE: usize = 5;

    let moving_average = SumTreeSMA::<f64, f64, WINDOW_SIZE>::new();
    let mutex = Arc::new(Mutex::new(moving_average));
    let mutex_cloned = Arc::clone(&mutex);

    tokio::spawn(async move {
        let mut system = System::new_all();
        loop {
            system.refresh_processes(ProcessesToUpdate::Some(&[current_pid]), false);
            if let Some(process) = system.process(current_pid) {
                let result = process.cpu_usage();
                mutex_cloned.lock().await.add_sample(result as f64);
            }
            sleep(cpu_collection_interval_ms).await;
        }
    });

    cpu_meter_provider.meter_provider
        .meter(DEFAULT_METER_NAME)
        .f64_observable_gauge("process_cpu_used")
        .with_callback(move |observer| {
            match mutex.try_lock() {
                Ok(moving_average) => {
                    let average_cpu_usage = moving_average.get_average();
                    observer.observe(average_cpu_usage, &[]);
                }
                Err(_) => {
                    eprintln!("Could not lock mutex that collects average cpu usage.");
                }
            }
        })
        .build();

    trace!("Initialized telemetry collection of OS metrics.");
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
