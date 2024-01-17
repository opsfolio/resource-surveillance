use clap::Parser;
use opentelemetry::trace::Tracer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cli::Cli::parse();

    if let Some(tracer) = service_management::start(&cli)? {
        let span = tracer.start("main");
        cmd::execute(&cli).await?;
        drop(span);
    } else {
        cmd::execute(&cli).await?;
    }

    Ok(())
}
