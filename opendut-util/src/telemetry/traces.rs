use opentelemetry::KeyValue;
use opentelemetry::trace::TraceError;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{Resource, runtime};
use opendut_auth::confidential::blocking::client::{ConfClientArcMutex, ConfidentialClientRef};
use crate::telemetry::opentelemetry_types::Endpoint;

pub(crate) fn init_tracer(telemetry_interceptor: ConfClientArcMutex<Option<ConfidentialClientRef>>, endpoint: &Endpoint, service_name: impl Into<String>, service_instance_id: impl Into<String>) -> Result<opentelemetry_sdk::trace::Tracer, TraceError> {
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
