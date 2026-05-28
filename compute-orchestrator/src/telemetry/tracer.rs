use opentelemetry::global;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::TracerProvider;
use opentelemetry_sdk::Resource;
use tracing::info;

pub fn init_tracer(service_name: &str, otlp_endpoint: Option<&str>) {
    let resource = Resource::new(vec![KeyValue::new(
        "service.name",
        service_name.to_string(),
    )]);

    let provider = if let Some(endpoint) = otlp_endpoint {
        match opentelemetry_otlp::SpanExporter::builder()
            .with_tonic()
            .with_endpoint(endpoint.to_string())
            .build()
        {
            Ok(exporter) => TracerProvider::builder()
                .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                .with_resource(resource)
                .build(),
            Err(e) => {
                tracing::warn!("Failed to build OTLP exporter: {}, using noop", e);
                TracerProvider::builder().with_resource(resource).build()
            }
        }
    } else {
        info!("No OTLP endpoint configured, using noop tracer");
        TracerProvider::builder().with_resource(resource).build()
    };

    let _ = global::set_tracer_provider(provider);
    info!("OpenTelemetry tracer initialized for '{}'", service_name);
}

pub fn get_tracer(name: &str) -> impl opentelemetry::trace::Tracer {
    global::tracer(name.to_string())
}
