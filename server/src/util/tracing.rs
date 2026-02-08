use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider};
use tracing_loki::url;
use tracing_subscriber::Layer;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::util::env::Var;
use crate::var;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

fn create_loki_layer(
    service_name: &str,
    loki_url: &str,
) -> Result<(
    impl Layer<tracing_subscriber::Registry>,
    impl std::future::Future<Output = ()>,
)> {
    let (layer, task) = tracing_loki::builder()
        .label("service_name", service_name)?
        .label("environment", "development")?
        .extra_field("pid", format!("{}", std::process::id()))?
        .build_url(url::Url::parse(loki_url)?)?;

    Ok((layer, task))
}

pub async fn build_subscriber() -> Result<trace::SdkTracerProvider> {
    let api_service_name = var!(Var::ApiServiceName).await?;
    let otelcol_endpoint = var!(Var::OtelExporterEndpoint).await?;
    let api_tracer_name = var!(Var::ApiTracerName).await?;
        
    // NOTE: 
    //  if debugging without telemetry collection, use `init_stdout_provider`
    //  over `init_provider`:
    //
    // ```
    // let provider = init_stdout_provider()?;
    // ```
    let provider = init_provider(api_service_name, otelcol_endpoint)?;
    let tracer = global::tracer(api_tracer_name);

    let (loki_layer, loki_task) = create_loki_layer(api_service_name, otelcol_endpoint)?;

    tracing_subscriber::registry()
        .with(loki_layer)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(EnvFilter::new(
            "server=debug,tower_http=debug,axum=debug,sqlx=info,irc=debug,info",
        ))
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(true)
                .with_line_number(true),
        )
        .init();

    tokio::spawn(loki_task);

    Ok(provider)
}

/// Intended for development purposes to enable tracing + logging to console without 
/// requiring external OTEL collection
#[allow(dead_code)]
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

fn init_provider(service_name: &'static str, endpoint: &str) -> Result<trace::SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(endpoint)
        .with_timeout(Duration::from_secs(5))
        .build()?;

    let provider = opentelemetry_sdk::trace::SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_resource(
            Resource::builder()
                .with_attributes([
                    KeyValue::new("service.name", service_name),
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
        eprintln!("error during tracer provider shutdown: {err:#?}");
    }
}
