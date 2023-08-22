use std::collections::HashMap;

use anyhow::Error;
use opentelemetry::{
    runtime::Tokio,
    sdk::{trace::Tracer, Resource},
    trace::TraceError,
    KeyValue,
};
use opentelemetry_otlp::WithExportConfig;
use reqwest::Client;
use tracing_log::env_logger;
use tracing_subscriber::{filter, fmt, layer::SubscriberExt, EnvFilter, Layer, Registry};

const HONEYCOMB_API_ENDPOINT: &str = "https://api.honeycomb.io/v1/traces";

fn init_tracer(api_key: &str) -> Result<Tracer, TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .http()
                .with_endpoint(HONEYCOMB_API_ENDPOINT)
                .with_http_client(Client::default())
                .with_headers(HashMap::from([("x-honeycomb-team".into(), api_key.into())])),
        )
        .with_trace_config(
            opentelemetry::sdk::trace::config().with_resource(Resource::new([KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                "rinha",
            )])),
        )
        .install_batch(Tokio)
}

const EXCLUDED_TARGETS: [&str; 1] = ["hyper"];

pub fn configure_tracing(api_key: &str) -> Result<(), Error> {
    env_logger::try_init()?;
    let tracer = init_tracer(api_key)?;
    let opentelemetry_layer = tracing_opentelemetry::layer()
        .with_tracer(tracer)
        .with_filter(filter::filter_fn(|metadata| {
            !EXCLUDED_TARGETS
                .into_iter()
                .any(|target| metadata.target().starts_with(target))
        }));

    let fmt_layer = fmt::layer().with_target(false);

    let subscriber = Registry::default()
        .with(EnvFilter::from_default_env())
        .with(fmt_layer)
        .with(opentelemetry_layer);

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
