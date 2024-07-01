use std::sync::Arc;
use std::time::Duration;
use opentelemetry::KeyValue;
use opentelemetry::metrics::{MeterProvider, MetricsError};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::{Resource, runtime};
use simple_moving_average::{SMA, SumTreeSMA};
use sysinfo::{Pid, System};
use tokio::sync::Mutex;
use tokio::time::sleep;
use opendut_auth::confidential::blocking::client::{ConfClientArcMutex, ConfidentialClientRef};
use crate::telemetry::DEFAULT_METER_NAME;
use crate::telemetry::opentelemetry_types::Endpoint;

pub fn init_metrics(telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>, endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>, metrics_interval: Duration) -> Result<SdkMeterProvider, MetricsError> {
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
