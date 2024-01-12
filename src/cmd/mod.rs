use std::path::PathBuf;

use autometrics::autometrics;
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::Serialize;

use sql_page::SQLPageArgs;
use crate::service_management;

pub mod admin;
pub mod capexec;
pub mod ingest;
pub mod notebooks;
pub mod sql_page;

const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";
const DEFAULT_MERGED_STATEDB_FS_PATH: &str = "resource-surveillance-aggregated.sqlite.db";

#[derive(Debug, Clone, Copy, ValueEnum, Default, Serialize)]
pub enum LogMode {
    Full,
    Json,
    #[default]
    Compact,
}

impl From<LogMode> for service_management::logger::LoggingMode {
    fn from(mode: LogMode) -> Self {
        match mode {
            LogMode::Full => service_management::logger::LoggingMode::Full,
            LogMode::Json => service_management::logger::LoggingMode::Json,
            LogMode::Compact => service_management::logger::LoggingMode::Compact,
        }
    }
}

#[derive(Debug, Serialize, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// How to identify this device
    #[arg(long, num_args = 0..=1, default_value = super::DEVICE.name(), default_missing_value = "always", env="SURVEILR_DEVICE_NAME")]
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
#[derive(Debug, Serialize, Subcommand)]
pub enum CliCommands {
    Admin(AdminArgs),
    CapturableExec(CapturableExecArgs),
    Ingest(IngestArgs),
    Notebooks(NotebooksArgs),
    #[clap(name = "sqlpage")]
    SQLPage(SQLPageArgs),
}

/// Admin / maintenance utilities
#[derive(Debug, Serialize, Args)]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommands,
}

#[derive(Debug, Serialize, Subcommand)]
pub enum AdminCommands {
    /// initialize an empty database with bootstrap.sql
    Init {
        /// target SQLite database
        #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
        state_db_fs_path: String,

        /// one or more globs to match as SQL files and batch execute them in alpha order
        #[arg(short = 'I', long)]
        state_db_init_sql: Vec<String>,

        /// remove the existing database first
        #[arg(short, long)]
        remove_existing_first: bool,

        /// add the current device in the empty database's device table
        #[arg(long)]
        with_device: bool,
    },

    /// merge multiple surveillance state databases into a single one
    Merge {
        /// one or more DB name globs to match and merge
        #[arg(short, long, default_value = "*.db")]
        candidates: Vec<String>,

        /// one or more DB name globs to ignore if they match
        #[arg(short = 'i', long)]
        ignore_candidates: Vec<String>,

        /// target SQLite database with merged content
        #[arg(short='d', long, default_value = DEFAULT_MERGED_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_MERGED_STATEDB_FS_PATH")]
        state_db_fs_path: String,

        /// one or more globs to match as SQL files and batch execute them in alpha order
        #[arg(short = 'I', long)]
        state_db_init_sql: Vec<String>,

        /// remove the existing database first
        #[arg(short, long)]
        remove_existing_first: bool,

        /// only generate SQL and emit to STDOUT (no actual merge)
        #[arg(long)]
        sql_only: bool,
    },

    /// generate CLI help markdown
    CliHelpMd,

    /// generate CLI help markdown
    Test(AdminTestArgs),
}

/// Capturable Executables (CE) assurance tools
#[derive(Debug, Serialize, Args)]
pub struct AdminTestArgs {
    #[command(subcommand)]
    pub command: AdminTestCommands,
}

#[derive(Debug, Serialize, Subcommand)]
pub enum AdminTestCommands {
    /// test capturable executables files
    Classifiers {
        /// target SQLite database
        #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
        state_db_fs_path: String,

        /// one or more globs to match as SQL files and batch execute them in alpha order
        #[arg(short = 'I', long)]
        state_db_init_sql: Vec<String>,

        /// only show the builtins, not from the database
        #[arg(long)]
        builtins: bool,
    },
}

/// Capturable Executables (CE) maintenance tools
#[derive(Debug, Serialize, Args)]
pub struct CapturableExecArgs {
    #[command(subcommand)]
    pub command: CapturableExecCommands,
}

#[derive(Debug, Serialize, Subcommand)]
pub enum CapturableExecCommands {
    /// list potential capturable executables
    Ls {
        /// one or more root paths to ingest
        #[arg(short, long, default_value = ".", default_missing_value = "always")]
        root_fs_path: Vec<String>,

        /// emit the results as markdown, not a simple table
        #[arg(long)]
        markdown: bool,
    },

    /// test capturable executables files
    Test(CapturableExecTestArgs),
}

/// Capturable Executables (CE) assurance tools
#[derive(Debug, Serialize, Args)]
pub struct CapturableExecTestArgs {
    #[command(subcommand)]
    pub command: CapturableExecTestCommands,
}

#[derive(Debug, Serialize, Subcommand)]
pub enum CapturableExecTestCommands {
    /// test capturable executables files
    File {
        #[arg(short, long)]
        fs_path: String,
    },

    /// Execute a task string as if it was run by `ingest tasks` and show the output
    Task {
        /// send commands in via STDIN the same as with `ingest tasks` and just emit the output
        #[arg(short, long)]
        stdin: bool,

        /// one or more commands that would work as a Deno Task line
        #[arg(short, long)]
        task: Vec<String>,

        /// use this as the current working directory (CWD)
        #[arg(long)]
        cwd: Option<String>,
    },
}

/// Ingest content from device file system and other sources
#[derive(Debug, Serialize, Args)]
pub struct IngestArgs {
    #[command(subcommand)]
    pub command: IngestCommands,
}

/// Ingest content from device file system and other sources
#[derive(Debug, Serialize, Args)]
pub struct IngestFilesArgs {
    /// don't run the ingestion, just report statistics
    #[arg(long)]
    pub dry_run: bool,

    /// the behavior name in `behavior` table
    #[arg(short, long, env = "SURVEILR_INGEST_BEHAVIOR_NAME")]
    pub behavior: Option<String>,

    /// one or more root paths to ingest
    #[arg(short, long, default_value = ".", default_missing_value = "always")]
    pub root_fs_path: Vec<String>,

    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    /// one or more globs to match as SQL files and batch execute them in alpha order
    #[arg(short = 'I', long)]
    pub state_db_init_sql: Vec<String>,

    /// include the surveil database in the ingestion candidates
    #[arg(long)]
    pub include_state_db_in_ingestion: bool,

    /// show stats as an ASCII table after completion
    #[arg(long)]
    pub stats: bool,

    /// show stats in JSON after completion
    #[arg(long)]
    pub stats_json: bool,

    /// save the options as a new behavior
    #[arg(long)]
    pub save_behavior: Option<String>,
}

/// Notebooks maintenance utilities
#[derive(Debug, Serialize, Args)]
pub struct IngestTasksArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    /// one or more globs to match as SQL files and batch execute them in alpha order
    #[arg(short = 'I', long)]
    pub state_db_init_sql: Vec<String>,

    /// read tasks from STDIN
    #[arg(long)]
    pub stdin: bool,

    /// show session stats after completion
    #[arg(long)]
    pub stats: bool,

    /// show session stats as JSON after completion
    #[arg(long)]
    pub stats_json: bool,
}

/// Ingest uniform resources content from multiple sources
#[allow(clippy::large_enum_variant)]
#[derive(Debug, Serialize, Subcommand)]
pub enum IngestCommands {
    Files(IngestFilesArgs),
    Tasks(IngestTasksArgs),
}

/// Notebooks maintenance utilities
#[derive(Debug, Serialize, Args)]
pub struct NotebooksArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: Option<String>,

    /// one or more globs to match as SQL files and batch execute them in alpha order
    #[arg(short = 'I', long)]
    state_db_init_sql: Vec<String>,

    #[command(subcommand)]
    pub command: NotebooksCommands,
}

#[derive(Debug, Serialize, Subcommand)]
pub enum NotebooksCommands {
    /// Notebooks' cells emit utilities
    Cat {
        /// search for these notebooks (include % for LIKE otherwise =)
        #[arg(short, long)]
        notebook: Vec<String>,

        /// search for these cells (include % for LIKE otherwise =)
        #[arg(short, long)]
        cell: Vec<String>,

        /// add separators before each cell
        #[arg(short, long)]
        seps: bool,
    },

    /// list all notebooks
    Ls {
        /// list all SQL cells that will be handled by execute_migrations
        #[arg(short, long)]
        migratable: bool,
    },
}

impl CliCommands {
    #[autometrics]
    pub async fn execute(&self, cli: &Cli) -> anyhow::Result<()> {
        match self {
            CliCommands::Admin(args) => args.command.execute(cli, args),
            CliCommands::CapturableExec(args) => args.command.execute(cli, args),
            CliCommands::Ingest(args) => args.command.execute(cli, args),
            CliCommands::Notebooks(args) => args.command.execute(cli, args),
            CliCommands::SQLPage(args) => args.execute(args).await
        }
    }
}
