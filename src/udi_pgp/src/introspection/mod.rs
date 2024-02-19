
//! # UDI-PGP Introspection Module.
//!
//! This module provides status update on the internals of UDI-PGP at a particular time, It exposes the
//! processor configuration, suppliers and their details and query details for each query like the
//! start and endtime. This is basically a thin layer over the admin DB SQLite file used for state management.
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

use std::{
    fmt::Display, path::PathBuf, str::FromStr, sync::{Arc, Mutex}
};

use futures::{stream, Stream};
use pgwire::{
    api::{
        portal::Format,
        results::{DataRowEncoder, FieldInfo, QueryResponse, Response},
        Type,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
    messages::data::DataRow,
};
use rusqlite::{types::ValueRef, Connection, Error as RusqliteError, Rows, Statement};

use crate::parser::stmt::UdiPgpStatment;

mod error;

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



pub struct IntrospectionBackend {
    conn: Arc<Mutex<Connection>>,
}

impl IntrospectionBackend {
    pub fn new(path: &PathBuf) -> Result<Self, RusqliteError> {
        let conn = Connection::open(path)?;
        Ok(IntrospectionBackend {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    fn name_to_type(&self, name: &str) -> PgWireResult<Type> {
        match name.to_uppercase().as_ref() {
            "INT" | "INTEGER" => Ok(Type::INT8),
            "VARCHAR" => Ok(Type::VARCHAR),
            "TEXT" => Ok(Type::TEXT),
            "BINARY" => Ok(Type::BYTEA),
            "FLOAT" => Ok(Type::FLOAT8),
            "TIMESTAMPTZ" => Ok(Type::TIMESTAMPTZ),
            "UUID" => Ok(Type::UUID),
            "TEXT[]" => Ok(Type::VARCHAR_ARRAY),
            _ => Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "ERROR".to_owned(),
                "42846".to_owned(),
                format!("Unsupported data type: {name}"),
            )))),
        }
    }

    fn row_desc_from_stmt(
        &self,
        stmt: &Statement,
        format: &Format,
    ) -> PgWireResult<Vec<FieldInfo>> {
        stmt.columns()
            .iter()
            .enumerate()
            .map(|(idx, col)| {
                let field_type = self.name_to_type(col.decl_type().unwrap())?;
                Ok(FieldInfo::new(
                    col.name().to_owned(),
                    None,
                    None,
                    field_type,
                    format.format_for(idx),
                ))
            })
            .collect()
    }

    fn encode_row_data(
        &self,
        mut rows: Rows,
        schema: Arc<Vec<FieldInfo>>,
    ) -> impl Stream<Item = PgWireResult<DataRow>> {
        let mut results = Vec::new();
        let ncols = schema.len();
        while let Ok(Some(row)) = rows.next() {
            let mut encoder = DataRowEncoder::new(schema.clone());
            for idx in 0..ncols {
                let data = row.get_ref_unwrap::<usize>(idx);
                match data {
                    ValueRef::Null => encoder.encode_field(&None::<i8>).unwrap(),
                    ValueRef::Integer(i) => {
                        encoder.encode_field(&i).unwrap();
                    }
                    ValueRef::Real(f) => {
                        encoder.encode_field(&f).unwrap();
                    }
                    ValueRef::Text(t) => {
                        encoder
                            .encode_field(&String::from_utf8_lossy(t).as_ref())
                            .unwrap();
                    }
                    ValueRef::Blob(b) => {
                        encoder.encode_field(&b).unwrap();
                    }
                }
            }

            results.push(encoder.finish());
        }

        stream::iter(results)
    }

    pub fn do_query<'a>(&self, stmt: &UdiPgpStatment) -> PgWireResult<Vec<Response<'a>>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare(&stmt.query)
            .map_err(|e| PgWireError::ApiError(Box::new(e)))?;
        let header = Arc::new(self.row_desc_from_stmt(&stmt, &Format::UnifiedText)?);
        stmt.query(())
            .map(|rows| {
                let s = self.encode_row_data(rows, header.clone());
                vec![Response::Query(QueryResponse::new(header, s))]
            })
            .map_err(|e| PgWireError::ApiError(Box::new(e)))
    }
}
