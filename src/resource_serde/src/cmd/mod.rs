use clap::{Args, Subcommand};
use serde::Serialize;

use self::imap::IngestImapArgs;

const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";
const DEFAULT_MERGED_STATEDB_FS_PATH: &str = "resource-surveillance-aggregated.sqlite.db";

pub mod imap;
pub mod transform;

/// Admin / maintenance utilities
#[derive(Debug, Serialize, Args, Clone)]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommands,
}

    #[derive(Debug, Serialize, Subcommand, Clone)]
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

    /// emit credentials
    Credentials(CredentialArgs),
}

/// Credentials for several services used in surveilr
#[derive(Debug, Serialize, Args, Clone)]
pub struct CredentialArgs {
    #[command(subcommand)]
    pub command: CredentialsCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum CredentialsCommands {
    #[clap(name = "microsoft-365")]
    /// microsoft 365 credentials
    Microsoft365 {
        /// Client ID of the application from MSFT Azure App Directory
        #[arg(short = 'i', long)]
        client_id: String,
        /// Client Secret of the application from MSFT Azure App Directory
        #[arg(short = 's', long)]
        client_secret: String,
        /// Redirect URL. Base redirect URL path. It gets concatenated with the server address to form the full redirect url,
        /// when using the `auth_code` mode for token generation.
        #[arg(short = 'r', long)]
        redirect_uri: Option<String>,
        /// Emit values to stdout
        #[arg(long)]
        env: bool,
        /// Emit values to stdout with the "export" syntax right in front to enable direct sourcing
        #[arg(long)]
        export: bool,
    },
}

/// Capturable Executables (CE) assurance tools
#[derive(Debug, Serialize, Args, Clone)]
pub struct AdminTestArgs {
    #[command(subcommand)]
    pub command: AdminTestCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
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
#[derive(Debug, Serialize, Args, Clone)]
pub struct CapturableExecArgs {
    #[command(subcommand)]
    pub command: CapturableExecCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
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
#[derive(Debug, Serialize, Args, Clone)]
pub struct CapturableExecTestArgs {
    #[command(subcommand)]
    pub command: CapturableExecTestCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
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
#[derive(Debug, Serialize, Args, Clone)]
pub struct IngestArgs {
    #[command(subcommand)]
    pub command: IngestCommands,
}

/// Ingest content from device file system and other sources
#[derive(Debug, Serialize, Args, Clone)]
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
#[derive(Debug, Serialize, Args, Clone)]
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
#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum IngestCommands {
    Files(IngestFilesArgs),
    Tasks(IngestTasksArgs),
    Imap(IngestImapArgs),
}

/// Notebooks maintenance utilities
#[derive(Debug, Serialize, Args, Clone)]
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

#[derive(Debug, Serialize, Subcommand, Clone)]
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

/// Configuration to start the SQLPage webserver
#[derive(Debug, Serialize, Args, Clone)]
pub struct SQLPageArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    /// Base URL for SQLPage to start from. Defaults to "/index.sql".
    #[arg(
        short = 'u',
        long,
        default_value = "/",
        default_missing_value = "always"
    )]
    pub url_base_path: String,

    /// Port to bind sqplage webserver to
    #[arg(short = 'p', long)]
    pub port: u16,

    /// Port that any OTEL compatible service is running on.
    #[arg(short = 'o', long)]
    pub otel: Option<u16>,

    /// Metrics port. Used for scraping metrics with tools like OpenObserve or Prometheus
    #[arg(short = 'm', long)]
    pub metrics: Option<u16>,
}
