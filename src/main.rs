use clap::Parser;
use opentelemetry::trace::Tracer;

#[macro_use]
extern crate lazy_static;

mod device;
lazy_static! {
    static ref DEVICE: device::Device = device::Device::new(None);
}

#[macro_use]
mod helpers;

mod cmd;
mod format;
mod frontmatter;
mod ingest;
mod models_polygenix;
mod persist;
mod resource;
mod shell;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cmd::Cli::parse();

    match (&cli.log_mode, &cli.debug, &cli.log_file) {
        (_, _, Some(file)) => utils::logger::log(
            cli.debug.into(),
            cli.log_mode.unwrap_or_default().into(),
            Some(file),
        )?,
        (_, _, None) => utils::logger::log(
            cli.debug.into(),
            cli.log_mode.unwrap_or_default().into(),
            None,
        )?,
    };

    let tracer = utils::otel::init()?;
    let tracer = tracer.inner();

    let span = tracer.start("main");

    cli.command.execute(&cli).await?;

    drop(span);

    Ok(())
}
