//! # UDI-PGP Introspection Module.
//!
//! This module provides status update on the internals of UDI-PGP at a particular time, It exposes the
//! processor configuration, suppliers and their details and query details for each query like the
//! start and endtime.
//!
//! ## Usage
//! - Select all current suppliers
//! ```sql
//! SELECT * FROM udi_pgp_supplier; -- Show existing suppliers
//! ```
//! - Show metrics port, current address UDI-PGP is bound to and the health port
//! ```sql
//! SELeCT * FROM udi_pgp_config; -- Show config entries
//! ```
//! - Query observabilty
//! ```sql
//! SELECT query_id, query_text, exec_status, exec_msg, elaboration, exec_start_at, exec_finish_at FROM udi_pgp_observe_query_exec; -- Show log entries, at start of surveilr it should be empty
//! ```

use std::{fmt::Display, str::FromStr};

use pgwire::api::results::FieldInfo;
use tokio::sync::mpsc;
use tracing::{debug, info};

use crate::{
    config::manager::Message,
    error::UdiPgpResult,
    introspection::sql_suppliers::{core_table, logs_table, suppliers_table},
    parser::stmt::UdiPgpStatment,
    sql_supplier::SqlSupplier,
    Row,
};
mod error;
mod sql_suppliers;

pub use self::error::IntrospectionError;

#[derive(Debug)]
pub enum IntrospectionTable {
    Supplier,
    Config,
    QueryExec,
}

impl FromStr for IntrospectionTable {
    type Err = IntrospectionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
          "udi_pgp_supplier" => Ok(IntrospectionTable::Supplier), 
          "udi_pgp_config" => Ok(IntrospectionTable::Config),
          "udi_pgp_observe_query_exec" => Ok(IntrospectionTable::QueryExec),
            other => {
                Err(IntrospectionError::TableError(format!(
                    "Expected one of `udi_pgp_supplier`, `udi_pgp_observe_query_exec`, `udi_pgp_config`. Got: {}",
                    other
                )))
            }
      }
    }
}

impl Display for IntrospectionTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntrospectionTable::Supplier => f.write_str("udi_pgp_supplier"),
            IntrospectionTable::Config => f.write_str("udi_pgp_config"),
            IntrospectionTable::QueryExec => f.write_str("udi_pgp_observe_query_exec"),
        }
    }
}

#[derive(Debug)]
pub struct Introspection<'a> {
    stmt: &'a UdiPgpStatment,
    state_tx: mpsc::Sender<Message>,
    table_type: IntrospectionTable,
}

impl<'a> Introspection<'a> {
    /// Checks the table in the statement to decide what section a query falls under
    /// - `udi_pgp_supplier` -> Suppliers Introspection
    /// - `udi_pgp_config` -> Core Introspection
    /// - `udi_pgp_observe_query_exec` -> Query and Logs introspection
    pub fn new(
        stmt: &'a UdiPgpStatment,
        tx: mpsc::Sender<Message>,
    ) -> Result<Self, IntrospectionError> {
        if stmt.tables.len() > 1 {
            return Err(IntrospectionError::TableError(format!(
                "UDI-PGP Introspection queries currently supports just one table. Got: {:?}",
                stmt.tables
            )));
        }

        let table_name = stmt
            .tables
            .first()
            .ok_or(IntrospectionError::TableError(
                "Table list is empty. Execute a query like: ` SELECT * FROM udi_pgp_supplier;`"
                    .to_string(),
            ))?
            .as_str();
        let table_type = IntrospectionTable::from_str(table_name)?;
        Ok(Introspection {
            stmt,
            state_tx: tx,
            table_type,
        })
    }

    pub async fn handle(&mut self) -> UdiPgpResult<(Vec<FieldInfo>, Vec<Vec<Row>>)> {
        info!("Executing Introspection Query");
        debug!("{:#?}", self.stmt);

        let res = match self.table_type {
            IntrospectionTable::Supplier => {
                let mut supp = suppliers_table::SupplierTable::new(self.state_tx.clone());
                let mut stmt = self.stmt.clone();
                let schema = supp.schema(&mut stmt).await?;
                let rows = supp.execute(&stmt).await?;
                (schema, rows)
            }
            IntrospectionTable::Config => {
                let mut supp = core_table::CoreTable::new(self.state_tx.clone());
                let mut stmt = self.stmt.clone();
                let schema = supp.schema(&mut stmt).await?;
                let rows = supp.execute(&stmt).await?;
                (schema, rows)
            }
            IntrospectionTable::QueryExec => {
                let mut supp = logs_table::LogTable::new(self.state_tx.clone());
                let mut stmt = self.stmt.clone();
                let schema = supp.schema(&mut stmt).await?;
                let rows = supp.execute(&stmt).await?;
                (schema, rows)
            }
        };

        Ok(res)
    }
}
