use std::sync::Arc;
use std::time::Duration;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{MeterProvider};
use opentelemetry_otlp::{MetricExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::metrics::{MetricError, SdkMeterProvider};
use opentelemetry_sdk::{Resource, runtime};
use simple_moving_average::{SMA, SumTreeSMA};
use sysinfo::{Pid, ProcessesToUpdate, System};
use tokio::sync::Mutex;
use tokio::time::sleep;
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use crate::telemetry::DEFAULT_METER_NAME;
use crate::telemetry::opentelemetry_types::Endpoint;
use opentelemetry_sdk::metrics::PeriodicReader;
use tracing::trace;

pub(super) fn init_metrics(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_name: impl Into<String>,
    service_instance_id: impl Into<String>,
    metrics_interval: Duration
) -> Result<SdkMeterProvider, MetricError> {

    let exporter = MetricExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(metrics_interval)
        .build();

    let provider = SdkMeterProvider::builder()
        .with_reader(reader)
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
            let average_cpu_usage = mutex.try_lock().unwrap().get_average();
            observer.observe(average_cpu_usage, &[]);
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
