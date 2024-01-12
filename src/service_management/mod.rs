use crate::cmd::Cli;
use opentelemetry_sdk::trace::{self};

pub mod logger;
mod observability;

pub fn start(cli: &Cli) -> anyhow::Result<Option<trace::Tracer>> {
    match (&cli.log_mode, &cli.debug, &cli.log_file) {
        (_, _, Some(file)) => logger::log(
            cli.debug.into(),
            cli.log_mode.unwrap_or_default().into(),
            Some(file),
        )?,
        (_, _, None) => logger::log(
            cli.debug.into(),
            cli.log_mode.unwrap_or_default().into(),
            None,
        )?,
    };

    let tracer = match &cli.command {
        crate::cmd::CliCommands::SQLPage(args) => {
            if let Some(port) = args.metrics {
                observability::init_metrics(port)?;
            }
            match args.otel {
                None => None,
                Some(port) => {
                    let tracer = observability::init_tracing(port)?;
                    Some(tracer.inner().clone())
                }
            }
        }
        _ => None,
    };

    Ok(tracer)
}
