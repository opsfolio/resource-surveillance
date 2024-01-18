use clap::Parser;
use opentelemetry::trace::Tracer;
use surveilr_cli::service_management;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = resource_serde::cmd::Cli::parse();

    if let Some(tracer) = service_management::start(&cli)? {
        let span = tracer.start("main");
        surveilr_cli::execute(&cli).await?;
        drop(span);
    } else {
        surveilr_cli::execute(&cli).await?;
    }

    Ok(())
}
