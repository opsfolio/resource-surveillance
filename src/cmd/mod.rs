use clap::{Args, Parser, Subcommand};
use regex::Regex;
use rusqlite::{Connection, OpenFlags};

use self::fswalk::fs_walk;
use crate::persist::*;

pub mod admin;
pub mod fswalk;
pub mod notebooks;

const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";
const DEFAULT_MERGED_STATEDB_FS_PATH: &str = "resource-surveillance-aggregated.sqlite.db";

#[derive(Parser)]
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
}

#[derive(Subcommand)]
pub enum CliCommands {
    Admin(AdminArgs),
    Notebooks(NotebooksArgs),
    FsWalk(FsWalkArgs),
}

/// Walks the device file system
#[derive(Args)]
pub struct FsWalkArgs {
    /// the behavior name in `behavior` table
    #[arg(short, long, env = "SURVEILR_FS_WALK_BEHAVIOR_NAME")]
    pub behavior: Option<String>,

    /// one or more root paths to walk
    #[arg(short, long, default_value = ".", default_missing_value = "always")]
    pub root_path: Vec<String>,

    /// reg-exes to use to ignore files in root-path(s)
    #[arg(
        short,
        long,
        default_value = "/(\\.git|node_modules)/",
        default_missing_value = "always"
    )]
    pub ignore_entry: Vec<Regex>,

    /// reg-exes to use to compute digests for
    #[arg(long, default_value = ".*", default_missing_value = "always")]
    pub compute_digests: Vec<Regex>,

    /// reg-exes to use to load content for entry instead of just walking
    #[arg(
        long,
        default_values_t = [
            Regex::new(r"\.(md|mdx|html|json|jsonc|toml|yaml)$").unwrap(),
            // if you don't want capturable executables stored in uniform_resource, remove the following
            Regex::new(r"surveilr\[(?P<nature>[^\]]*)\]").unwrap()],
        default_missing_value = "always"
    )]
    pub surveil_content: Vec<Regex>,

    /// reg-exes to use to execute and capture STDOUT, STDERR (e.g. *.surveilr[json].sh) with "nature" capture group
    #[arg(
        long,
        // if you want capturable executables stored in uniform_resource, be sure it's also in surveil_content
        default_value = r"surveilr\[(?P<nature>[^\]]*)\]",
        default_missing_value = "always"
    )]
    pub capture_exec: Vec<regex::Regex>,

    /// reg-exes that will signify which captured executables' output should be treated as batch SQL
    #[arg(
        long,
        // if you want capturable executables stored in uniform_resource, be sure it's also in surveil_content
        default_value = r"surveilr-SQL",
        default_missing_value = "always"
    )]
    pub captured_exec_sql: Vec<regex::Regex>,

    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    /// include the surveil database in the walk
    #[arg(long)]
    pub include_state_db_in_walk: bool,

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
#[derive(Args)]
pub struct NotebooksArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: Option<String>,

    #[command(subcommand)]
    pub command: NotebooksCommands,
}

#[derive(Subcommand)]
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

/// Admin / maintenance utilities
#[derive(Args)]
pub struct AdminArgs {
    #[command(subcommand)]
    pub command: AdminCommands,
}

#[derive(Subcommand)]
pub enum AdminCommands {
    /// initialize an empty database with bootstrap.sql
    Init {
        /// target SQLite database
        #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
        state_db_fs_path: String,

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

        /// remove the existing database first
        #[arg(short, long)]
        remove_existing_first: bool,

        /// only generate SQL and emit to STDOUT (no actual merge)
        #[arg(long)]
        sql_only: bool,
    },

    /// generate CLI help markdown
    CliHelpMd,
}

impl CliCommands {
    pub fn execute(&self, cli: &Cli) -> anyhow::Result<()> {
        match self {
            CliCommands::FsWalk(args) => match fs_walk(cli, args) {
                Ok(walk_session_id) => {
                    if args.stats || args.stats_json {
                        if let Ok(conn) = Connection::open_with_flags(
                            args.state_db_fs_path.clone(),
                            OpenFlags::SQLITE_OPEN_READ_WRITE,
                        ) {
                            if args.stats_json {
                                if let Ok(stats) = fs_content_walk_session_stats_latest(
                                    &conn,
                                    walk_session_id.clone(),
                                ) {
                                    print!("{}", serde_json::to_string_pretty(&stats).unwrap())
                                }
                            }

                            if args.stats {
                                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                                fs_walk_session_stats(
                                    &conn,
                                    |_index,
                                     root_path,
                                     file_extension,
                                     file_count,
                                     with_content_count,
                                     with_frontmatter_count| {
                                        if args.root_path.len() < 2 {
                                            rows.push(vec![
                                                file_extension,
                                                file_count.to_string(),
                                                with_content_count.to_string(),
                                                with_frontmatter_count.to_string(),
                                            ]);
                                        } else {
                                            rows.push(vec![
                                                root_path,
                                                file_extension,
                                                file_count.to_string(),
                                                with_content_count.to_string(),
                                                with_frontmatter_count.to_string(),
                                            ]);
                                        }
                                        Ok(())
                                    },
                                    walk_session_id,
                                )
                                .unwrap();
                                println!(
                                    "{}",
                                    if args.root_path.len() < 2 {
                                        crate::format::format_table(
                                            &["Extn", "Count", "Content", "Frontmatter"],
                                            &rows,
                                        )
                                    } else {
                                        crate::format::format_table(
                                            &["Path", "Extn", "Count", "Content", "Frontmatter"],
                                            &rows,
                                        )
                                    }
                                );
                            }
                        } else {
                            println!(
                                "Notebooks cells command requires a database: {}",
                                args.state_db_fs_path
                            );
                        }
                    }
                    Ok(())
                }
                Err(err) => Err(err),
            },
            CliCommands::Notebooks(args) => args.command.execute(cli, args),
            CliCommands::Admin(args) => args.command.execute(cli, args),
        }
    }
}
