use std::{collections::HashMap, sync::Arc};

use clap::{Args, Subcommand};
use serde::Serialize;
use tokio::sync::Mutex;
use udi_pgp::{
    auth::Auth,
    config::{Supplier, UdiPgpSshTarget},
    error::UdiPgpResult,
    sql_supplier::SqlSupplierType,
    UdiPgpModes,
};
use udi_pgp_osquery::OsquerySupplier;

/// UDI PostgreSQL Proxy for remote SQL starts up a server which pretends to be PostgreSQL
/// but proxies its SQL to other CLI services with SQL-like interface (called SQL Suppliers).
#[derive(Debug, Serialize, Args, Clone)]
pub struct PgpArgs {
    /// IP address to bind udi-pgp to.
    #[arg(short = 'a', long, default_value = "127.0.0.1:5432")]
    pub addr: std::net::SocketAddr,

    /// Username for authentication
    #[arg(short = 'u', long)]
    pub username: String,

    /// Password for authentication
    #[arg(short = 'p', long)]
    pub password: String,

    /// Identification for the supplier which will be passed to the client. e.g
    /// surveilr udi pgp -u john -p doe -i test-supplier osquery local
    /// The psql comand will be: psql -h 127.0.0.1 -p 5432 -d "test-supplier" -c "select * from system_info"
    #[arg(short = 'i', long)]
    pub supplier_id: String,

    #[command(subcommand)]
    pub command: PgpCommands,
}

#[derive(Debug, Serialize, Subcommand, Clone)]
pub enum PgpCommands {
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
    /// execute osquery on remote hosts
    Remote {
        /// SSH details of hosts to execute osquery on including and identifier. e,g. "user@127.0.0.1:22,john"/"user@host.com:1234,doe"
        #[arg(short = 's', long)]
        ssh_targets: Vec<String>,
    },
}

impl PgpArgs {
    pub async fn execute(&self) -> anyhow::Result<()> {
        let auth = Auth::new(&self.username, &self.password);
        let (supplier, config_supplier) = self.create_supplier(&self.command, auth)?;

        let mut config_suppliers = HashMap::new();
        config_suppliers.insert(self.supplier_id.to_string(), config_supplier);

        let config = udi_pgp::config::UdiPgpConfig::new(self.addr, config_suppliers)?;
        let mut suppliers = HashMap::new();
        suppliers.insert(self.supplier_id.to_string(), Arc::new(Mutex::new(supplier)));

        udi_pgp::run(Arc::new(config), suppliers).await
    }

    fn create_supplier(
        &self,
        command: &PgpCommands,
        auth: Auth,
    ) -> anyhow::Result<(SqlSupplierType, Supplier)> {
        match command {
            PgpCommands::Osquery(OsqueryArgs { command }) => match command {
                OsqueryCommands::Local { atc_file_path } => {
                    let mode = UdiPgpModes::Local;
                    let supplier = Supplier::new(
                        "osquery",
                        mode.clone(),
                        None,
                        atc_file_path.clone(),
                        vec![auth],
                    );
                    Ok((
                        Box::new(OsquerySupplier::new(mode).with_atc_file(atc_file_path)),
                        supplier,
                    ))
                }
                OsqueryCommands::Remote { ssh_targets } => {
                    let mode = UdiPgpModes::Remote;
                    let targets = ssh_targets
                        .iter()
                        .map(UdiPgpSshTarget::try_from)
                        .collect::<UdiPgpResult<Vec<_>>>()?;
                    let supplier =
                        Supplier::new("osquery", mode.clone(), Some(targets), None, vec![auth]);
                    Ok((
                        Box::new(OsquerySupplier::new(mode).with_ssh_targets(ssh_targets.to_vec())),
                        supplier,
                    ))
                }
            },
        }
    }
}
