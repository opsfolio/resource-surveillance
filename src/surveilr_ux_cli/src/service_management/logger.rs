use std::{
    fs::File,
    io,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use tracing::Level;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

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

pub fn log(
    debug_level: Verbosity,
    _mode: LoggingMode,
    log_file: Option<&PathBuf>,
) -> anyhow::Result<()> {
    let level: Level = debug_level.into();
    let env_filter = EnvFilter::new(level.to_string());

    let log_file = log_file.cloned();
    let log_file = Arc::new(RwLock::new(log_file));

    let writer_factory = move || -> Box<dyn io::Write + Send + Sync> {
        let log_file = log_file.read().expect("RwLock read lock failed");
        match &*log_file {
            Some(path) => Box::new(File::create(path).expect("Failed to create log file")),
            None => Box::new(io::stdout()),
        }
    };

    let fmt_layer = fmt::layer()
        .compact()
        .with_line_number(true)
        .with_writer(writer_factory);

    let subscriber = Registry::default().with(env_filter).with(fmt_layer);

    let _guard = subscriber.set_default();

    Ok(())
}
