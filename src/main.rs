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
mod service_management;
mod shell;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cmd::Cli::parse();

    if let Some(tracer) = service_management::start(&cli)? {
        let span = tracer.start("main");
        cli.command.execute(&cli).await?;
        drop(span);
    } else {
        cli.command.execute(&cli).await?;
    }

    Ok(())
}
