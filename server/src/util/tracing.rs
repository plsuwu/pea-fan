use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider};
use tracing::instrument::WithSubscriber;
use tracing_loki::url;
use tracing_subscriber::Layer;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::util::env::Var;
use crate::var;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

pub const SERVICE_NAME: &str = "piss-fan-api";
pub const TRACER_NAME: &str = "pissfan-tracer";
pub const LOKI_URL: &str = "http://localhost:3100";

fn create_loki_layer(
    service_name: &str,
    loki_url: &str,
) -> Result<(
    impl Layer<tracing_subscriber::Registry>,
    impl std::future::Future<Output = ()>,
)> {
    use tracing_loki::BackgroundTask;
    let (layer, task) = tracing_loki::builder()
        .label("service_name", service_name)?
        .label("environment", "development".to_string())?
        .extra_field("pid", format!("{}", std::process::id()))?
        .build_url(url::Url::parse(loki_url)?)?;

    Ok((layer, task))
}

pub async fn build_subscriber() -> Result<trace::SdkTracerProvider> {
    // let provider = init_provider()?;
    let provider = init_stdout_provider()?;
    let tracer = global::tracer(TRACER_NAME);

    let (loki_layer, loki_task) = create_loki_layer(SERVICE_NAME, LOKI_URL)?;

    tracing_subscriber::registry()
        .with(loki_layer)
        .with(EnvFilter::new(
            "server=trace,tower_http=debug,axum=debug,sqlx=info,irc=trace,info",
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .init();

    tokio::spawn(loki_task);

    Ok(provider)
}

fn init_stdout_provider() -> Result<trace::SdkTracerProvider> {
    let exporter = opentelemetry_stdout::SpanExporter::default();
    let provider = SdkTracerProvider::builder()
        .with_simple_exporter(exporter)
        .with_id_generator(RandomIdGenerator::default())
        .with_sampler(Sampler::AlwaysOn)
        .build();

    global::set_tracer_provider(provider.clone());
    Ok(provider)
}

fn init_provider() -> Result<trace::SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint("http://localhost:4318")
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
