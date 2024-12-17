use opentelemetry::KeyValue;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::{SpanExporter, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::{Resource, runtime};
use opentelemetry_sdk::trace::TracerProvider;
use opendut_auth::confidential::blocking::client::{ConfClientArcMutex, ConfidentialClientRef};
use crate::telemetry::opentelemetry_types::Endpoint;

pub(crate) fn init_tracer(
    telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>,
    endpoint: &Endpoint,
    service_name: impl Into<String>,
    service_instance_id: impl Into<String>
) -> Result<TracerProvider, TraceError> {

    let exporter = SpanExporter::builder()
        .with_tonic()
        .with_interceptor(telemetry_interceptor)
        .with_tls_config(tonic::transport::ClientTlsConfig::new().with_enabled_roots())
        .with_endpoint(Clone::clone(&endpoint.url))
        .build()?;

    let provider = TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_resource(
            Resource::new(vec![
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                    service_name.into()),
                KeyValue::new(
                    opentelemetry_semantic_conventions::resource::SERVICE_INSTANCE_ID,
                    service_instance_id.into()
                )
            ])
        )
        .build();

    Ok(provider)
}
