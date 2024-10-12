use opentelemetry::{global::{self}, KeyValue, trace::TracerProvider};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    runtime,
    trace::{BatchConfig, RandomIdGenerator, Sampler, Tracer},
    Resource,
};
use opentelemetry_semantic_conventions::{
    resource::{DEPLOYMENT_ENVIRONMENT_NAME, SERVICE_NAME, SERVICE_VERSION,},
    SCHEMA_URL,
};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource(service_name: String) -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, service_name),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT_NAME, "develop"),
        ],
        SCHEMA_URL,
    )
}

pub fn setup_otel(collector_uri: &str, service_name: &str) -> OtelGuard {
    // collector_uri.clone().map_or_else(
    //     || {
    //         println!("No collector url found.");
    //     },
    //     |url| { 
    //       println!("setting url: {url:?}");
  let tracer = init_tracer(collector_uri, service_name);
    //     },
    // );

    let filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::builder().from_env_lossy()
    } else {
        format!(
            "info",
        )
        .parse()
        .expect("valid EnvFilter value can be parsed")
    };

    tracing_subscriber::registry()
        .with(filter) // Read global subscriber filter from `RUST_LOG`
        .with(tracing_subscriber::fmt::layer()) // Setup logging layer
        .with(OpenTelemetryLayer::new(tracer))
        .init();

    OtelGuard
}

// Construct Tracer for OpenTelemetryLayer
fn init_tracer(url: &str, service_name: impl Into<String>) -> Tracer {
    let tracer_provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                // Customize sampling strategy
                .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                    1.0,
                ))))
                // If export trace to AWS X-Ray, you can use XrayIdGenerator
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource(service_name.into())),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(url),
        )
        .with_batch_config(BatchConfig::default())
        .install_batch(runtime::Tokio)
        .expect("opentelemetry tracer to configure correctly");

      global::set_tracer_provider(tracer_provider.clone());

      tracer_provider.tracer("tracing-otel-subscriber")
}

pub struct OtelGuard;

impl Drop for OtelGuard {
    fn drop(&mut self) {
        opentelemetry::global::shutdown_tracer_provider();
    }
}