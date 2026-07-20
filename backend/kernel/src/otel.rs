use opentelemetry::global;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init_tracing(service_name: &str) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    let fmt_layer = tracing_subscriber::fmt::layer().with_target(true).json();

    if tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer)
        .try_init()
        .is_err()
    {
        tracing::debug!(service = service_name, "tracing already initialized");
        return;
    }

    tracing::info!(service = service_name, "tracing initialized");
}

pub fn shutdown_tracer_provider() {
    global::shutdown_tracer_provider();
}
