use crate::telemetry::opentelemetry_types::Endpoint;
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::{runtime, Resource};

pub(crate) fn init_tracer(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_metadata_resource: Resource,
) -> Result<TracerProvider, TraceError> {

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(service_metadata_resource)
        .build();

    Ok(provider)
}
