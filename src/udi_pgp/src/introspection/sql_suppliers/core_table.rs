use async_trait::async_trait;
use derive_new::new;
use pgwire::api::{
    results::{FieldFormat, FieldInfo},
    Type,
};
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    config::{Supplier, SupplierType, UdiPgpConfig},
    error::{UdiPgpError, UdiPgpResult},
    parser::stmt::{ColumnMetadata, ExpressionType, UdiPgpStatment},
    sql_supplier::{SqlSupplier, SqlSupplierType},
    state::messages::Message,
    Row,
};

#[derive(Debug, Clone, new)]
pub struct CoreTable {
    state_tx: mpsc::Sender<Message>,
    query_session_id: Option<Uuid>,
}

impl CoreTable {
    fn fill_columns_with_default(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<()> {
        let cols: Vec<&str> = vec![
            "addr",
            "health",
            "metrics",
            "surveilr_version",
            "admin_db_path",
        ];
        stmt.columns = cols
            .iter()
            .map(|col| {
                Ok(ColumnMetadata::new(
                    col.to_string(),
                    ExpressionType::Standard,
                    None,
                    Type::VARCHAR,
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
                    Type::VARCHAR,
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
impl SqlSupplier for CoreTable {
    fn name(&self) -> &str {
        "suppliers_table"
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
        let config = self.read_config().await?;
        let columns = &stmt.columns;

        let mut rows = Vec::with_capacity(3);
        let mut cell_row = Vec::with_capacity(columns.len());

        cell_row.push(Row::from(config.addr().to_string()));
        cell_row.push(Row::from(match config.metrics {
            Some(a) => a.to_string(),
            None => "null".to_string(),
        }));
        cell_row.push(Row::from(match config.health {
            Some(a) => a.to_string(),
            None => "null".to_string(),
        }));
        let surveilr_version = env!("CARGO_PKG_VERSION");
        cell_row.push(Row::from(surveilr_version.to_string()));

        let admindb_path = config.admin_state_fs_path.to_str().unwrap();
        cell_row.push(Row::from(admindb_path.to_string()));

        rows.push(cell_row);

        Ok(rows)
    }
}
