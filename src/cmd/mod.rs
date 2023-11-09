use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};
use regex::Regex;

use self::fswalk::fs_walk;

pub mod fswalk;
pub mod notebooks;

const DEFAULT_DB: &str = "./resource-surveillance.sqlite.db";

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Generate a Markdown file of all CLI commands and options
    #[arg(long)]
    pub help_markdown: bool,

    /// How to identify this device
    #[arg(long, num_args = 0..=1, default_value = super::DEVICE.name(), default_missing_value = "always")]
    pub device_name: Option<String>,

    /// TODO: Use a Deno *.ts or Nickel config file for defaults, allowing CLI args as overrides
    #[arg(short, long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// TODO: Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub debug: u8,

    #[command(subcommand)]
    pub command: CliCommands,
}

#[derive(Subcommand)]
pub enum CliCommands {
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
    #[arg(
            long,
            default_value = DEFAULT_DB,
            default_missing_value = "always"
        )]
    pub surveil_db_fs_path: String,
}

/// Notebooks maintenance utilities
#[derive(Args)]
pub struct NotebooksArgs {
    /// target SQLite database
    #[arg(
            long,
            default_value = DEFAULT_DB,
            default_missing_value = "always"
        )]
    pub surveil_db_fs_path: Option<String>,

    #[command(subcommand)]
    pub command: NotebooksCommands,
}

// TODO: separate commands
// - surveilr nb emit
// - surveilr nb cat
// - surveilr nb ls
// - surveilr nb run --table (--json is default)

#[derive(Subcommand)]
pub enum NotebooksCommands {
    /// Notebooks' cells emit utilities
    Cat {
        // search for these notebooks (include % for LIKE otherwise =)
        #[arg(short, long)]
        notebook: Vec<String>,

        // search for these cells (include % for LIKE otherwise =)
        #[arg(short, long)]
        cell: Vec<String>,
    },

    /// list all notebooks
    Ls,
}

impl CliCommands {
    pub fn execute(&self, cli: &Cli) -> anyhow::Result<()> {
        match self {
            CliCommands::FsWalk(args) => fs_walk(cli, args),
            CliCommands::Notebooks(args) => args.command.execute(cli, args),
        }
    }
}
