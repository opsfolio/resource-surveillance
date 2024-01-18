use clap::Parser;
use surveilr_cli::service_management;
use opentelemetry::trace::Tracer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = cmd::Cli::parse();

    if let Some(tracer) = service_management::start(&cli)? {
        let span = tracer.start("main");
        surveilr_cli::execute(&cli).await?;
        drop(span);
    } else {
        surveilr_cli::execute(&cli).await?;
    }

    Ok(())
}
