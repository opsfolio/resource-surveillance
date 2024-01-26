use std::{collections::HashMap, path::PathBuf, sync::Arc};

use anyhow::anyhow;
use clap::{Args, Subcommand};
use serde::Serialize;
use tokio::sync::Mutex;
use udi_pgp::{
    auth::Auth,
    config::{Supplier, SupplierType, UdiPgpConfig},
    error::UdiPgpResult,
    sql_supplier::{SqlSupplierMap, SqlSupplierType},
    ssh::UdiPgpSshTarget,
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
    pub username: Option<String>,

    /// Password for authentication
    #[arg(short = 'p', long)]
    pub password: Option<String>,

    /// Identification for the supplier which will be passed to the client. e.g
    /// surveilr udi pgp -u john -p doe -i test-supplier osquery local
    /// The psql comand will be: psql -h 127.0.0.1 -p 5432 -d "test-supplier" -c "select * from system_info"
    #[arg(short = 'i', long)]
    pub supplier_id: Option<String>,

    /// Config file for UDI-PGP. Either a .ncl file or JSON file
    #[arg(short = 'c', long)]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<PgpCommands>,
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
        let (config, suppliers) = if let Some(config_file) = &self.config {
            let config = UdiPgpConfig::try_from_file(config_file)?;
            let suppliers = self.suppliers_from_config(&config);
            (config, suppliers)
        } else if let Some(pgp_command) = &self.command {
            self.try_config_from_args(pgp_command)?
        } else {
            return Err(anyhow!("Either a subcommand or a config file is required"));
        };

        udi_pgp::run(Arc::new(config), suppliers).await
    }

    fn suppliers_from_config(&self, config: &UdiPgpConfig) -> SqlSupplierMap {
        config
            .suppliers
            .iter()
            .map(|(k, v)| (k.to_string(), self.create_supplier_from_config(v)))
            .collect()
    }

    fn create_supplier_from_config(
        &self,
        config_supplier: &Supplier,
    ) -> Arc<Mutex<SqlSupplierType>> {
        match config_supplier.supplier_type {
            SupplierType::Osquery => Arc::new(Mutex::new(Box::new(OsquerySupplier::from(
                config_supplier,
            )) as SqlSupplierType)),
            SupplierType::Git => unimplemented!(),
        }
    }

    fn try_config_from_args(
        &self,
        commands: &PgpCommands,
    ) -> anyhow::Result<(UdiPgpConfig, SqlSupplierMap)> {

        let (username, password) = match (&self.username, &self.password) {
            (Some(u), Some(p)) => (u, p),
            _ => return Err(anyhow!("Authentication for supplier incomplete")),
        };
        let supplier_id = match &self.supplier_id {
            None => return Err(anyhow!("Supplier ID must be present")),
            Some(id) => id,
        };

        let auth = Auth::new(username, password);
        let (supplier, config_supplier) = self.create_supplier_from_args(commands, auth)?;

        let mut config_suppliers = HashMap::new();
        config_suppliers.insert(supplier_id.to_string(), config_supplier);

        let config = UdiPgpConfig::new(self.addr, config_suppliers)?;
        let mut suppliers = HashMap::new();
        suppliers.insert(supplier_id.to_string(), Arc::new(Mutex::new(supplier)));

        Ok((config, suppliers))
    }

    fn create_supplier_from_args(
        &self,
        command: &PgpCommands,
        auth: Auth,
    ) -> anyhow::Result<(SqlSupplierType, Supplier)> {
        match command {
            PgpCommands::Osquery(OsqueryArgs { command }) => match command {
                OsqueryCommands::Local { atc_file_path } => {
                    let mode = UdiPgpModes::Local;
                    let supplier = Supplier::new(
                        SupplierType::Osquery,
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
                    let supplier = Supplier::new(
                        SupplierType::Osquery,
                        mode.clone(),
                        Some(targets),
                        None,
                        vec![auth],
                    );
                    Ok((
                        Box::new(OsquerySupplier::new(mode).with_ssh_targets(ssh_targets.to_vec())),
                        supplier,
                    ))
                }
            },
        }
    }
}
