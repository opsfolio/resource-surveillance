use anyhow::anyhow;
use clap::{Args, Subcommand};
use serde::Serialize;

use crate::transformers::{HtmlTransformer, Transformer};

const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";

/// Resource transformation utilities for data stored in the RSSD.
#[derive(Debug, Serialize, Args, Clone)]
pub struct TransformArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    state_db_fs_path: String,

    /// Indicates if all current transforms should be deleted before running the transform.
    #[arg(short, long, default_value = "false")]
    reset_transforms: bool,

    #[command(subcommand)]
    pub command: TransformCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum TransformCommands {
    /// Transform HTML content
    Html {
        /// List of CSS selectors with names and values.
        /// e.g. -css-select="name_of_select_query:div > p"
        /// i.e, select all p tags in a div tag
        #[arg(short, long)]
        css_select: Vec<String>,
    },
    /// Transform markdown content
    Markdown {},
}

impl TransformArgs {
    pub fn transform(&self) -> anyhow::Result<()> {
        let transformer: Box<dyn Transformer> = match &self.command {
            TransformCommands::Html { css_select } => Box::new(HtmlTransformer::new(
                css_select.to_vec(),
                self.state_db_fs_path.clone(),
            )),

            _ => return Err(anyhow!("Unsupported")),
        };
        transformer.insert(self.reset_transforms)?;
        Ok(())
    }
}
