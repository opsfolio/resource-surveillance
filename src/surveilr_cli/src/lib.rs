use resource_serde::cmd::{Cli, CliCommands};

pub mod admin;
pub mod capexec;
pub mod ingest;
pub mod notebooks;
pub mod service_management;
pub mod sql_page;
pub mod udi_pgp;

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        CliCommands::Admin(args) => admin::Admin::default().execute(args, cli),
        CliCommands::CapturableExec(args) => capexec::CapturableExec::default().execute(cli, args),
        CliCommands::Ingest(args) => ingest::Ingest::default().execute(cli, args),
        CliCommands::Notebooks(args) => notebooks::Notebooks::default().execute(cli, args),
        CliCommands::SQLPage(args) => sql_page::SqlPage::default().execute(args).await,
        CliCommands::UdiPgp(args) => udi_pgp::execute(args).await,
    }
}
