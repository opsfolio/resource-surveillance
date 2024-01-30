use clap::{Args, Subcommand};
use serde::Serialize;

use self::pgp::PgpArgs;

pub mod pgp;

/// Universal Data Infrastructure
#[derive(Debug, Serialize, Args, Clone)]
pub struct UdiArgs {
    #[command(subcommand)]
    commands: UdiCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum UdiCommands {
    Pgp(PgpArgs),
    Admin,
}

impl UdiArgs {
    pub async fn execute(&self) -> anyhow::Result<()> {
        match &self.commands {
            UdiCommands::Pgp(args) => { 
                args.register_suppliers().await;
                args.execute().await },
            UdiCommands::Admin => Ok(()),
        }
    }
}
