use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use common::DEVICE;
use resource_serde::cmd::{AdminArgs, CapturableExecArgs, IngestArgs, NotebooksArgs, SQLPageArgs};
use serde::Serialize;
use udi::UdiArgs;

pub mod admin;
pub mod capexec;
pub mod ingest;
pub mod notebooks;
pub mod service_management;
pub mod sql_page;
pub mod udi;

#[derive(Debug, Clone, Copy, ValueEnum, Default, Serialize)]
pub enum LogMode {
    Full,
    Json,
    #[default]
    Compact,
}

#[derive(Debug, Serialize, Parser, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// How to identify this device
    #[arg(long, num_args = 0..=1, default_value = DEVICE.name(), default_missing_value = "always", env="SURVEILR_DEVICE_NAME")]
    pub device_name: Option<String>,

    /// Turn debugging information on (repeat for higher levels)
    #[arg(short, long, action = clap::ArgAction::Count, env="SURVEILR_DEBUG")]
    pub debug: u8,

    #[command(subcommand)]
    pub command: CliCommands,

    /// Output logs in json format.
    #[clap(long, value_enum)]
    pub log_mode: Option<LogMode>,

    /// File for logs to be written to
    #[arg(long, value_parser)]
    pub log_file: Option<PathBuf>,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum CliCommands {
    Admin(AdminArgs),
    CapturableExec(CapturableExecArgs),
    Ingest(IngestArgs),
    Notebooks(NotebooksArgs),
    #[clap(name = "sqlpage")]
    SQLPage(SQLPageArgs),
    #[clap(name = "udi")]
    Udi(UdiArgs),
}

pub async fn execute(cli: &Cli) -> anyhow::Result<()> {
    match &cli.command {
        CliCommands::Admin(args) => admin::Admin::default().execute(args, cli),
        CliCommands::CapturableExec(args) => capexec::CapturableExec::default().execute(cli, args),
        CliCommands::Ingest(args) => ingest::Ingest::default().execute(cli, args),
        CliCommands::Notebooks(args) => notebooks::Notebooks::default().execute(cli, args),
        CliCommands::SQLPage(args) => sql_page::SqlPage::default().execute(args).await,
        CliCommands::Udi(args) => args.execute().await,
    }
}
