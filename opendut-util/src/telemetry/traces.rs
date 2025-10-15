use crate::telemetry::opentelemetry_types::Endpoint;
use opendut_auth::confidential::client::{ConfClientArcMutex, ConfidentialClientRef};
use opentelemetry_otlp::{ExporterBuildError, SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::{SdkTracerProvider};
use opentelemetry_sdk::Resource;
use tonic::transport::ClientTlsConfig;

pub(crate) fn init_tracer(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_metadata_resource: Resource,
    tls_config: ClientTlsConfig,
) -> Result<SdkTracerProvider, ExporterBuildError> {

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_tls_config(tls_config)
        .with_interceptor(telemetry_interceptor)
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let tracer_provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(service_metadata_resource)
        .build();
    opentelemetry::global::set_tracer_provider(tracer_provider.clone());

    Ok(tracer_provider)
}
