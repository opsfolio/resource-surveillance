use std::{fs::File, path::PathBuf, sync::Arc};

use tracing::{subscriber, trace, Level};
use tracing_subscriber::{
    fmt::{
        format::{Compact, DefaultFields, Format, Json, JsonFields, Pretty},
        SubscriberBuilder,
    },
    EnvFilter, FmtSubscriber,
};

#[derive(Debug)]
pub enum Verbosity {
    Info,
    Debug,
    Trace,
}

impl From<u8> for Verbosity {
    fn from(v: u8) -> Self {
        match v {
            0 => Verbosity::Info,
            1 => Verbosity::Debug,
            _ => Verbosity::Trace,
        }
    }
}

impl From<Verbosity> for Level {
    fn from(v: Verbosity) -> Self {
        match v {
            Verbosity::Info => Level::INFO,
            Verbosity::Debug => Level::DEBUG,
            Verbosity::Trace => Level::TRACE,
        }
    }
}

pub enum LoggingMode {
    Full,
    Json,
    Compact,
}

fn standard_fmt(level: Level) -> SubscriberBuilder {
    FmtSubscriber::builder()
        .with_max_level(level)
        .with_thread_ids(true)
        .with_thread_names(true)
        .with_line_number(true)
        .with_file(true)
}

fn compact_fmt(level: Level) -> SubscriberBuilder<DefaultFields, Format<Compact>> {
    FmtSubscriber::builder()
        .with_max_level(level)
        .with_line_number(false)
        .with_file(false)
        .compact()
}

fn full_fmt(level: Level) -> SubscriberBuilder<Pretty, Format<Pretty>> {
    standard_fmt(level).pretty()
}

fn json_fmt(level: Level) -> SubscriberBuilder<JsonFields, Format<Json>> {
    standard_fmt(level).json().flatten_event(true)
}

pub fn log(
    debug_level: Verbosity,
    mode: LoggingMode,
    log_file: Option<&PathBuf>,
) -> anyhow::Result<()> {
    let verbosity = debug_level;
    let level: Level = verbosity.into();
    let env_filter = EnvFilter::builder()
        .with_default_directive(level.into())
        .from_env()?;

    let s = match mode {
        LoggingMode::Compact => {
            let subscriber = compact_fmt(level).with_env_filter(env_filter);
            if let Some(file) = log_file {
                let debug_log = match File::create(file) {
                    Ok(file) => Arc::new(file),
                    Err(_) => {
                        eprintln!("Failed to create file: {:#?}", file);
                        return Ok(subscriber::set_global_default(subscriber.finish())?);
                    }
                };

                subscriber::set_global_default(subscriber.with_writer(debug_log).finish())
            } else {
                subscriber::set_global_default(subscriber.finish())
            }
        }
        LoggingMode::Json => {
            let subscriber = json_fmt(level).with_env_filter(env_filter);

            if let Some(file) = log_file {
                let debug_log = match File::create(file) {
                    Ok(file) => Arc::new(file),
                    Err(_) => {
                        eprintln!("Failed to create file: {:#?}", file);
                        return Ok(subscriber::set_global_default(subscriber.finish())?);
                    }
                };

                subscriber::set_global_default(subscriber.with_writer(debug_log).finish())
            } else {
                subscriber::set_global_default(subscriber.finish())
            }
        }
        LoggingMode::Full => {
            let subscriber = full_fmt(level).with_env_filter(env_filter);

            if let Some(file) = log_file {
                let debug_log = match File::create(file) {
                    Ok(file) => Arc::new(file),
                    Err(_) => {
                        eprintln!("Failed to create file: {:#?}", file);
                        return Ok(subscriber::set_global_default(subscriber.finish())?);
                    }
                };

                subscriber::set_global_default(subscriber.with_writer(debug_log).finish())
            } else {
                subscriber::set_global_default(subscriber.finish())
            }
        }
    };
    trace!(set_level = %level, "log level set");

    Ok(s?)
}
