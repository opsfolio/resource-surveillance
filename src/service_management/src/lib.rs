use cli::{Cli, CliCommands, LogMode};
use opentelemetry_sdk::trace::{self};

pub mod logger;
mod observability;

impl From<LogMode> for logger::LoggingMode {
    fn from(mode: LogMode) -> Self {
        match mode {
            LogMode::Full => logger::LoggingMode::Full,
            LogMode::Json => logger::LoggingMode::Json,
            LogMode::Compact => logger::LoggingMode::Compact,
        }
    }
}

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
        CliCommands::SQLPage(args) => {
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
