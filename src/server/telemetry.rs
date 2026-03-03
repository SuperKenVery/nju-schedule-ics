//! Setup opentelementary and grafana

use anyhow::{Context, Result};
use base64::Engine as _;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider as _;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::trace::{BatchConfigBuilder, BatchSpanProcessor};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

/// Initialises the global tracing subscriber.
///
/// When all three OTel config values are present, a second layer exports spans
/// to the given OTLP endpoint in addition to the usual stdout fmt output.
/// Uses `try_init` so that repeated calls in dev-mode hot-reloads are silently
/// ignored instead of panicking.
pub async fn init(
    otel_endpoint: Option<&str>,
    otel_instance_id: Option<&str>,
    otel_token: Option<&str>,
) -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));
    let fmt_layer = tracing_subscriber::fmt::layer();

    match (otel_endpoint, otel_instance_id, otel_token) {
        (Some(endpoint), Some(instance_id), Some(token)) => {
            let auth = format!(
                "Basic {}",
                base64::engine::general_purpose::STANDARD
                    .encode(format!("{}:{}", instance_id, token))
            );

            let mut headers = std::collections::HashMap::new();
            headers.insert("Authorization".to_string(), auth);

            let traces_endpoint = format!("{}/v1/traces", endpoint.trim_end_matches('/'));

            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_http()
                .with_protocol(Protocol::HttpBinary)
                .with_endpoint(&traces_endpoint)
                .with_headers(headers)
                .build()
                .context("Failed to build OTel OTLP span exporter")?;

            let resource = Resource::new(vec![KeyValue::new("service.name", "nju-schedule-ics")]);

            let processor =
                BatchSpanProcessor::builder(exporter, opentelemetry_sdk::runtime::Tokio)
                    .with_batch_config(
                        BatchConfigBuilder::default()
                            .with_scheduled_delay(std::time::Duration::from_secs(2))
                            .with_max_export_timeout(std::time::Duration::from_secs(10))
                            .build(),
                    )
                    .build();

            let provider = opentelemetry_sdk::trace::TracerProvider::builder()
                .with_span_processor(processor)
                .with_resource(resource)
                .build();

            opentelemetry::global::set_tracer_provider(provider.clone());
            opentelemetry::global::set_text_map_propagator(
                opentelemetry_sdk::propagation::TraceContextPropagator::new(),
            );
            let tracer = provider.tracer("nju-schedule-ics");

            let res = tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .with(tracing_opentelemetry::layer().with_tracer(tracer))
                .try_init();
            if let Err(error) = res {
                tracing::warn!(
                    "Failed to init tracing subscriber for opentelementry: {:#?}",
                    error
                );
            } else {
                tracing::info!(
                    endpoint = %traces_endpoint,
                    "OpenTelemetry tracing enabled → Grafana Cloud"
                );
            }
        }
        _ => {
            let _ = tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer)
                .try_init();
        }
    }

    Ok(())
}
