use std::time::Duration;

use opentelemetry::{KeyValue, global};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::{self, Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::metrics::SdkMeterProvider;
use opentelemetry_sdk::trace::{self, RandomIdGenerator, Sampler, SdkTracerProvider};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::util::env::Var;
use crate::var;

pub type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Telemetry {
    pub tracer_name: &'static str,
    pub base_resource: Resource,
    pub collector_url: &'static str,

    logger_provider: SdkLoggerProvider,
    tracer_provider: SdkTracerProvider,
    meter_provider: SdkMeterProvider,
}

impl Telemetry {
    pub async fn new() -> Result<Telemetry> {
        let collector_url = var!(Var::OtelExporterEndpoint).await?;
        let tracer_name = var!(Var::ApiTracerName).await?;
        let service_name = var!(Var::ApiServiceName).await?;
        let service_version = env!("CARGO_PKG_VERSION");

        let base_resource = base_attrs(service_name, service_version);

        let logger_provider = build_logger_provider(collector_url, base_resource.clone())?;
        let meter_provider = build_meter_provider(collector_url, base_resource.clone())?;
        let tracer_provider = build_tracer_provider(collector_url, base_resource.clone())?;

        Ok(Self {
            base_resource,
            tracer_name,
            collector_url,
            logger_provider,
            tracer_provider,
            meter_provider,
        })
    }

    pub fn register(self) -> Self {
        global::set_tracer_provider(self.tracer_provider.clone());
        let tracer = global::tracer(self.tracer_name);
        let trace_layer = tracing_opentelemetry::layer().with_tracer(tracer);

        let log_layer = OpenTelemetryTracingBridge::new(&self.logger_provider);
        let meter_layer = tracing_opentelemetry::MetricsLayer::new(self.meter_provider.clone());

        tracing_subscriber::registry()
            .with(trace_layer)
            .with(log_layer)
            .with(meter_layer)
            .with(EnvFilter::new(
                "piss_fan_server=debug,tower_http=debug,axum=debug,sqlx=info,info",
            ))
            .with(
                tracing_subscriber::fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_line_number(true),
            )
            .init();

        self
    }

    pub fn shutdown(self) {
        if let Err(e) = self.meter_provider.shutdown() {
            eprintln!("error during metering shutdown: {e:?}");
        } else {
            println!("metering shut down ok");
        }

        if let Err(e) = self.logger_provider.shutdown() {
            eprintln!("error during logging shutdown: {e:?}");
        } else {
            println!("logging shut down ok");
        }

        if let Err(e) = self.tracer_provider.shutdown() {
            eprintln!("error during tracing shutdown: {e:?}");
        } else {
            println!("tracing shut down ok");
        }
    }
}

pub fn build_logger_provider(
    collector_url: &str,
    base_resource: Resource,
) -> Result<SdkLoggerProvider> {
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(Endpoint::Logs.to_url(collector_url))
        .with_timeout(Duration::from_secs(5))
        .build()?;

    Ok(SdkLoggerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(base_resource.clone())
        .build())
}

pub fn build_tracer_provider(
    collector_url: &str,
    base_resource: Resource,
) -> Result<SdkTracerProvider> {
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(Endpoint::Traces.to_url(collector_url))
        .with_timeout(Duration::from_secs(5))
        .build()?;

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_resource(base_resource.clone())
        .build();

    global::set_tracer_provider(provider.clone());

    Ok(provider)
}

pub fn build_meter_provider(
    collector_url: &str,
    base_resource: Resource,
) -> Result<SdkMeterProvider> {
    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_protocol(Protocol::Grpc)
        .with_endpoint(Endpoint::Metrics.to_url(collector_url))
        .with_timeout(Duration::from_secs(5))
        .build()?;

    Ok(SdkMeterProvider::builder()
        .with_periodic_exporter(exporter)
        .with_resource(base_resource.clone())
        .build())
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

fn base_attrs(name: &'static str, version: &'static str) -> Resource {
    Resource::builder()
        .with_attributes([
            KeyValue::new("service.name", name),
            KeyValue::new("service.version", version),
        ])
        .build()
}

enum Endpoint {
    Logs,
    Traces,
    Metrics,
}

impl Endpoint {
    pub fn to_url(&self, collector_endpoint: &str) -> String {
        let location: &str = match self {
            Endpoint::Logs => "/v1/logs",
            Endpoint::Traces => "/v1/traces",
            Endpoint::Metrics => "/v1/metrics",
        };
        format!("{collector_endpoint}{location}")
    }
}
