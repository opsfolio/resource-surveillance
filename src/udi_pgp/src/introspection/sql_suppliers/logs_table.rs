use async_trait::async_trait;
use derive_new::new;
use pgwire::api::{
    results::{FieldFormat, FieldInfo},
    Type,
};
use tokio::sync::{mpsc, oneshot};
use tracing::error;
use uuid::Uuid;

use crate::{
    config::{manager::Message, Supplier, SupplierType},
    error::{UdiPgpError, UdiPgpResult},
    observability::QueryLogEntryMap,
    parser::stmt::{ColumnMetadata, ExpressionType, UdiPgpStatment},
    sql_supplier::{SqlSupplier, SqlSupplierType},
    Row,
};

#[derive(Debug, Clone, new)]
pub struct LogTable {
    state_tx: mpsc::Sender<Message>,
    query_session_id: Option<Uuid>,
}

impl LogTable {
    fn convert_to_pgwire_type(&self, col: &str) -> UdiPgpResult<Type> {
        match col {
            "query_id" | "query_text" => Ok(Type::VARCHAR),
            "exec_finish_at" | "exec_start_at" => Ok(Type::TIMESTAMP),
            "exec_status" => Ok(Type::INT4),
            "exec_msg" => Ok(Type::ANYARRAY),
            "elaboration" => Ok(Type::JSON),
            other => Err(UdiPgpError::SchemaError(
                other.to_string(),
                "Column name invalid".to_string(),
            )),
        }
    }

    fn fill_columns_with_default(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<()> {
        let cols: Vec<&str> = vec![
            "query_id",
            "query_text",
            "exec_start_at",
            "exec_finish_at",
            "elaboration",
            "exec_msg",
            "exec_status",
        ];
        stmt.columns = cols
            .iter()
            .map(|col| {
                Ok(ColumnMetadata::new(
                    col.to_string(),
                    ExpressionType::Standard,
                    None,
                    self.convert_to_pgwire_type(col)?,
                ))
            })
            .collect::<UdiPgpResult<Vec<_>>>()?;
        Ok(())
    }

    fn fill_columns_with_pgwire_types(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<()> {
        stmt.columns = stmt
            .columns
            .iter()
            .map(|col| {
                Ok(ColumnMetadata::new(
                    col.name.to_string(),
                    ExpressionType::Standard,
                    None,
                    self.convert_to_pgwire_type(&col.name)?,
                ))
            })
            .collect::<UdiPgpResult<Vec<_>>>()?;
        Ok(())
    }

    fn create_field_info_from_columns(&self, stmt: &UdiPgpStatment) -> Vec<FieldInfo> {
        stmt.columns
            .iter()
            .map(|col| {
                FieldInfo::new(
                    col.name.to_string(),
                    None,
                    None,
                    col.r#type.clone(),
                    FieldFormat::Text,
                )
            })
            .collect()
    }

    async fn read_log_entries(&self) -> UdiPgpResult<QueryLogEntryMap> {
        let (response_tx, response_rx) = oneshot::channel();
        let read_state_msg = Message::ReadLogEntries(response_tx);
        self.state_tx
            .send(read_state_msg)
            .await
            .expect("Failed to send message");
        match response_rx.await {
            Ok(logs) => Ok(logs),
            Err(e) => {
                error!("{}", e);
                Err(UdiPgpError::ConfigError(format!(
                    "Failed to read log entries: {}",
                    e
                )))
            }
        }
    }
}

#[async_trait]
impl SqlSupplier for LogTable {
    fn name(&self) -> &str {
        "log_entries_table"
    }

    fn supplier_type(&self) -> SupplierType {
        SupplierType::Introspection
    }

    fn update(&mut self, _supplier: Supplier) -> UdiPgpResult<()> {
        unimplemented!()
    }

    fn generate_new(&self, _supplier: Supplier) -> UdiPgpResult<SqlSupplierType> {
        unimplemented!()
    }

    fn add_session_id(&mut self, session_id: Uuid) -> UdiPgpResult<()> {
        self.query_session_id = Some(session_id);
        Ok(())
    }

    async fn schema(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<Vec<FieldInfo>> {
        if stmt.columns.len() == 1 && stmt.columns.first().unwrap().name == "*" {
            self.fill_columns_with_default(stmt)?;
        } else {
            self.fill_columns_with_pgwire_types(stmt)?;
        }

        let field_info = self.create_field_info_from_columns(stmt);

        Ok(field_info)
    }

    async fn execute(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<Vec<Row>>> {
        let entries = self.read_log_entries().await?;
        let columns = &stmt.columns;

        let mut rows = Vec::with_capacity(entries.len());
        for (_id, log) in entries.iter() {
            let mut cell_row = Vec::with_capacity(columns.len());
            for col in columns {
                let name = col.name.as_str();
                let row = match name {
                  "query_id" => log.query_id.to_string(),
                  "query_text" => log.query_text.to_string(),
                  "exec_start_at" => serde_json::to_string_pretty(&log.exec_start_at)?,
                  "exec_finish_at" => serde_json::to_string_pretty(&log.exec_finish_at)?,
                  "elaboration" => serde_json::to_string(&log.elaboration)?,
                  "exec_status" => {
                    if log.exec_msg.is_empty() {
                        0.to_string()
                    } else {
                        1.to_string()
                    }
                  }
                  "exec_msg" => serde_json::to_string(&log.exec_msg)?,
                  other => return Err(UdiPgpError::QueryExecutionError(format!("Invalid column name. Got: {}. Expected one of id, type, auth, ssh_target, atc_file_path", other)))
              };
                cell_row.push(Row::from(row));
            }
            rows.push(cell_row);
        }

        Ok(rows)
    }
}
