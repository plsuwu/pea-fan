use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub const SERVICE_NAME: &str = "piss-fan-api";
pub const TRACER_NAME: &str = "pissfan-tracer";

pub async fn build_subscriber() -> Result<trace::SdkTracerProvider> {
    let provider = init_provider()?;
    let tracer = global::tracer(TRACER_NAME);

    tracing_subscriber::registry()
        .with(EnvFilter::new(
            "server=trace,tower_http=debug,axum=debug,sqlx=info,irc=debug,info",
        ))
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .init();

    Ok(provider)
}

fn init_provider() -> Result<trace::SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_endpoint("http://localhost:4318/v1/traces")
        .with_timeout(Duration::from_secs(5))
        .build()?;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", SERVICE_NAME),
                    KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
                ])
                .build(),
        )
        .build();

    global::set_tracer_provider(provider.clone());
    Ok(provider)
}

pub fn destroy_tracer(provider: SdkTracerProvider) {
    if let Err(err) = provider.shutdown() {
        eprintln!("error during tracer provider shutdown: {:#?}", err);
    }
}
