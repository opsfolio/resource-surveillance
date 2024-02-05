use tracing::subscriber::set_global_default;
use tracing::{Event, Id, Level, Subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, EnvFilter, Layer, Registry};

struct UdiPgpTracingLayer {}

impl<S> Layer<S> for UdiPgpTracingLayer where S: Subscriber + for<'a> LookupSpan<'a> {}

pub fn init() -> anyhow::Result<()> {
    let env_filter = EnvFilter::new(Level::DEBUG.to_string());

    let fmt_layer = fmt::layer().compact().with_line_number(true);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(UdiPgpTracingLayer {}) // Assuming UdiPgpTracingLayer is your custom layer
        .with(fmt_layer);

    set_global_default(subscriber)?;
    Ok(())
}
