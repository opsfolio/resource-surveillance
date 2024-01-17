use std::time::Duration;

use opentelemetry::metrics::MeterProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_sdk::metrics::reader::{DefaultAggregationSelector, DefaultTemporalitySelector};
use opentelemetry_sdk::{
    trace::{self, RandomIdGenerator, Sampler},
    Resource,
};

use tracing::instrument::{WithDispatch, WithSubscriber};

pub fn init_tracing(port: u16) -> anyhow::Result<WithDispatch<trace::Tracer>> {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(format!("http://localhost:{port}"))
                .with_timeout(Duration::from_secs(3)),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "surveilr",
                )])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?
        .with_current_subscriber();

    Ok(tracer)
}

pub fn init_metrics(port: u16) -> anyhow::Result<()> {
    let export_config = ExportConfig {
        endpoint: format!("http://localhost:{port}").to_string(),
        timeout: Duration::from_secs(3),
        protocol: Protocol::HttpBinary,
    };

    let metrics_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_export_config(export_config),
        )
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "surveilr",
        )]))
        .with_period(Duration::from_secs(3))
        .with_timeout(Duration::from_secs(10))
        .with_aggregation_selector(DefaultAggregationSelector::new())
        .with_temporality_selector(DefaultTemporalitySelector::new())
        .build()?;

    metrics_provider.meter("surveilr");
    let _meter = metrics_provider.versioned_meter(
        "surveilr",
        Some(env!("CARGO_PKG_VERSION")),
        Some("https://opentelemetry.io/schema/1.0.0"),
        Some(vec![]),
    );

    let _log_exporter = opentelemetry_otlp::new_pipeline()
        .logging()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint("http://localhost:4317"),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)?
        .with_current_subscriber();
    Ok(())
}
