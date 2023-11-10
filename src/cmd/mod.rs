use clap::{Args, Parser, Subcommand};
use regex::Regex;

use self::fswalk::fs_walk;

pub mod admin;
pub mod fswalk;
pub mod notebooks;

const DEFAULT_DB: &str = "./resource-surveillance.sqlite.db";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// How to identify this device
    #[arg(long, num_args = 0..=1, default_value = super::DEVICE.name(), default_missing_value = "always")]
    pub device_name: Option<String>,

    /// Turn debugging information on (repeat for higher levels)
    #[arg(short, long, action = clap::ArgAction::Count)]
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
        default_value = "\\.(md|mdx|html|json|jsonc)$",
        default_missing_value = "always"
    )]
    pub surveil_content: Vec<Regex>,

    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_DB, default_missing_value = "always")]
    pub surveil_db_fs_path: String,
}

/// Notebooks maintenance utilities
#[derive(Args)]
pub struct NotebooksArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_DB, default_missing_value = "always")]
    pub surveil_db_fs_path: Option<String>,

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
    Ls,
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
        #[arg(short='d', long, default_value = DEFAULT_DB, default_missing_value = "always")]
        surveil_db_fs_path: String,

        /// remove the existing database first
        #[arg(short, long)]
        remove_existing_first: bool,
    },

    /// generate CLI help markdown
    CliHelpMd,
}

impl CliCommands {
    pub fn execute(&self, cli: &Cli) -> anyhow::Result<()> {
        match self {
            CliCommands::FsWalk(args) => fs_walk(cli, args),
            CliCommands::Notebooks(args) => args.command.execute(cli, args),
            CliCommands::Admin(args) => args.command.execute(cli, args),
        }
    }
}
