use async_trait::async_trait;
use derive_new::new;
use futures::future::join_all;
use pgwire::api::{
    results::{FieldFormat, FieldInfo},
    Type,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    config::{Supplier, SupplierType, UdiPgpConfig}, error::{UdiPgpError, UdiPgpResult}, parser::stmt::{ColumnMetadata, ExpressionType, UdiPgpStatment}, sql_supplier::{SqlSupplier, SqlSupplierType}, state::messages::Message, Row
};

#[derive(Debug, Clone, new)]
pub struct SupplierTable {
    state_tx: mpsc::Sender<Message>,
    query_session_id: Option<Uuid>,
}

impl SupplierTable {
    fn convert_to_pgwire_type(&self, col: &str) -> UdiPgpResult<Type> {
        match col {
            "id" | "type" | "mode" | "atc_file_path" => Ok(Type::VARCHAR),
            "ssh_targets" | "auth" => Ok(Type::JSON),
            other => Err(UdiPgpError::SchemaError(
                other.to_string(),
                "Column name invalid".to_string(),
            )),
        }
    }

    fn fill_columns_with_default(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<()> {
        let cols: Vec<&str> = vec!["id", "type", "mode", "ssh_targets", "atc_file_path", "auth"];
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

    async fn read_config(&self) -> UdiPgpResult<UdiPgpConfig> {
        let (response_tx, response_rx) = oneshot::channel();
        let read_state_msg = Message::ReadConfig(response_tx);
        self.state_tx
            .send(read_state_msg)
            .await
            .expect("Failed to send message");
        match response_rx.await {
            Ok(config) => {
                debug!("Latest Config: {:#?}", config);
                Ok(config)
            }
            Err(e) => {
                error!("{}", e);
                Err(UdiPgpError::ConfigError(format!(
                    "Failed to read configuration: {}",
                    e
                )))
            }
        }
    }
}

#[async_trait]
impl SqlSupplier for SupplierTable {
    fn name(&self) -> &str {
        "suppliers_table"
    }

    fn supplier_type(&self) -> SupplierType {
        SupplierType::Introspection
    }

    fn update(&mut self, _supplier: Supplier) -> UdiPgpResult<()> {
        unimplemented!()
    }

    fn add_session_id(&mut self, session_id: Uuid) -> UdiPgpResult<()> {
        self.query_session_id = Some(session_id);
        Ok(())
    }

    fn generate_new(&self, _supplier: Supplier) -> UdiPgpResult<SqlSupplierType> {
        unimplemented!()
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
        let config = self.read_config().await?;
        let mut suppliers = config.suppliers;
        let columns = &stmt.columns;

        let mut rows = Vec::with_capacity(suppliers.len());
        for (id, supplier) in suppliers.iter_mut() {
            let mut cell_row = Vec::with_capacity(columns.len());
            for col in columns {
                let name = col.name.as_str();
                let row = match name {
                  "id" => id.to_string(),
                  "type" => supplier.supplier_type.to_string(),
                  "mode" => supplier.mode.to_string(),
                  "atc_file_path" => {
                    match &supplier.atc_file_path {
                        Some(p) => p.to_string(),
                        None => "NULL".to_string()
                    }
                  }
                  "auth" => serde_json::to_string_pretty(&supplier.auth)?,
                  "ssh_targets" => {
                     if let Some(targets) = &mut supplier.ssh_targets {
                        let check_accessibility = targets.iter_mut().map(|target| target.is_accessible());
                        let _: Vec<UdiPgpResult<()>> = join_all(check_accessibility).await;
                        serde_json::to_string_pretty(targets)?
                    } else {
                        "[]".to_string()
                    }
                  },
                  other => return Err(UdiPgpError::QueryExecutionError(format!("Invalid column name. Got: {}. Expected one of id, type, auth, ssh_target, atc_file_path", other)))
              };
                cell_row.push(Row::from(row));
            }
            rows.push(cell_row);
        }

        Ok(rows)
    }
}
