use crate::telemetry::opentelemetry_types::Endpoint;
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use opentelemetry_otlp::{ExporterBuildError, SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::{SdkTracerProvider};
use opentelemetry_sdk::Resource;

pub(crate) fn init_tracer(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_metadata_resource: Resource,
) -> Result<SdkTracerProvider, ExporterBuildError> {

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(service_metadata_resource)
        .build();
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}
