use std::{pin::Pin, str::FromStr, sync::Arc};

use futures::{stream, Stream};
use pgwire::{
    api::{
        results::{DataRowEncoder, FieldInfo},
        MakeHandler,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
    messages::data::DataRow,
};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error};

use crate::{
    config::UdiPgpConfig,
    error::UdiPgpResult,
    parser::UdiPgpQueryParser,
    simulations::response,
    sql_supplier::{SqlSupplierMap, SqlSupplierType},
    Row,
};

pub mod query_handler;

#[derive(Debug, Clone)]
pub struct UdiPgpProcessor {
    query_parser: UdiPgpQueryParser,
    config: UdiPgpConfig,
    suppliers: Arc<RwLock<SqlSupplierMap>>,
}

impl UdiPgpProcessor {
    pub fn new(config: &UdiPgpConfig, suppliers: SqlSupplierMap) -> Self {
        UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
            config: config.clone(),
            suppliers: Arc::new(RwLock::new(suppliers)),
        }
    }

    pub async fn supplier(&self, identifier: &str) -> PgWireResult<Arc<Mutex<SqlSupplierType>>> {
        let suppliers = self.suppliers.read().await;
        suppliers.get(identifier).cloned().ok_or_else(|| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "PROCESSOR".to_string(),
                format!("Supplier not found. Got: {}", identifier),
            )))
        })
    }

    pub fn extract_supplier_and_database(
        &self,
        param: Option<&String>,
    ) -> PgWireResult<(String, Option<String>)> {
        let db = param.ok_or_else(|| {
            error!("Cannot find database parameter");
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "PROCESSOR".to_string(),
                "Cannot find database parameter".to_string(),
            )))
        })?;

        let parts: Vec<&str> = db.split(':').collect();

        let supplier = parts.first()
            .ok_or_else(|| {
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "01".to_string(),
                    "Supplier is absent".to_string(),
                )))
            })?
            .to_string();

        let identifier = parts.get(1).map(|s| s.to_string());

        Ok((supplier, identifier))
    }

    pub fn encode_rows(
        &self,
        schema: Arc<Vec<FieldInfo>>,
        rows: &[Vec<Row>],
    ) -> Pin<Box<dyn Stream<Item = PgWireResult<DataRow>> + Send + Sync>> {
        debug!("encoding rows");

        let mut results = Vec::new();
        let ncols = schema.len();

        rows.iter().for_each(|row| {
            let mut encoder = DataRowEncoder::new(schema.clone());
            for idx in 0..ncols {
                let data = &row.get(idx).unwrap().value;
                encoder.encode_field(&data).unwrap();
            }

            results.push(encoder.finish());
        });

        debug!("encoded rows successfully");
        Box::pin(stream::iter(results))
    }

    pub fn simulate_driver_responses(
        &self,
        query: &str,
    ) -> UdiPgpResult<(Vec<FieldInfo>, Vec<Vec<Row>>)> {
        let (schema, rows) = response::driver_queries_response(query)?;
        let rows = vec![rows
            .into_iter()
            .map(|r| Row::from_str(r).unwrap())
            .collect::<Vec<_>>()];
        Ok((schema, rows))
    }
}

impl MakeHandler for UdiPgpProcessor {
    type Handler = Arc<UdiPgpProcessor>;

    fn make(&self) -> Self::Handler {
        Arc::new(UdiPgpProcessor {
            query_parser: self.query_parser.clone(),
            config: self.config.clone(),
            suppliers: self.suppliers.clone(),
        })
    }
}
