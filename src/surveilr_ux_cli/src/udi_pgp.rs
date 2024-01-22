use clap::{Args, Subcommand};
use serde::Serialize;
use udi_pgp::{auth::Auth, sql_supplier::SqlSupplierType, UdiPgpModes};
use udi_pgp_osquery::OsquerySupplier;

/// UDI PostgreSQL Proxy for remote SQL starts up a server which pretends to be PostgreSQL
/// but proxies its SQL to other CLI services with SQL-like interface (called SQL Suppliers).
#[derive(Debug, Serialize, Args, Clone)]
pub struct UdiPgpArgs {
    /// IP address to bind udi-pgp to.
    #[arg(short = 'a', long, default_value = "127.0.0.1:5432")]
    pub addr: std::net::SocketAddr,

    /// Username for authentication
    #[arg(short = 'u', long)]
    pub username: String,

    /// Password for authentication
    #[arg(short = 'p', long)]
    pub password: String,

    #[command(subcommand)]
    pub command: UdiPgpCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum UdiPgpCommands {
    /// query a machine
    Osquery(OsqueryArgs),
}

/// Modes to execute osquery in
#[derive(Debug, Serialize, Args, Clone)]
pub struct OsqueryArgs {
    #[command(subcommand)]
    pub command: OsqueryCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum OsqueryCommands {
    /// execute osquery on the local machine
    Local {
        /// ATC Configuration File path
        #[arg(short = 'a', long)]
        atc_file_path: Option<String>,
    },
    /// execute osquery on a remote machine
    Remote,
}

impl UdiPgpArgs {
    pub async fn execute(&self) -> anyhow::Result<()> {
        let UdiPgpArgs {
            addr,
            username,
            password,
            command,
        } = self;

        let auth = Auth::new(username, password);
        let config = udi_pgp::config::UdiPgpConfig::new(*addr, auth);

        let supplier = match command {
            UdiPgpCommands::Osquery(OsqueryArgs { command }) => self.create_supplier(command),
        };

        udi_pgp::run(&config, supplier).await
    }

    fn create_supplier(&self, command: &OsqueryCommands) -> SqlSupplierType {
        match command {
            OsqueryCommands::Local { atc_file_path } => {
                Box::new(OsquerySupplier::new(UdiPgpModes::Local).with_atc_file(atc_file_path))
            }
            OsqueryCommands::Remote => Box::new(OsquerySupplier::new(UdiPgpModes::Remote)),
        }
    }
}
