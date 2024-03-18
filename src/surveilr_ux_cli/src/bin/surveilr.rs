use clap::Parser;
use opentelemetry::trace::Tracer;
use surveilr::{service_management, Cli};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if let Some(tracer) = service_management::start(&cli)? {
        let span = tracer.start("main");
        surveilr::execute(&cli).await?;
        drop(span);
    } else {
        surveilr::execute(&cli).await?;
    }

    Ok(())
}
