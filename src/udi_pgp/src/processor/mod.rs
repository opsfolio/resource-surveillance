use std::{pin::Pin, str::FromStr, sync::Arc};

use futures::{stream, Stream};
use pgwire::{
    api::{
        results::{DataRowEncoder, FieldInfo},
        MakeHandler,
    },
    error::PgWireResult,
    messages::data::DataRow,
};
use tokio::sync::Mutex;
use tracing::debug;

use crate::{
    config::UdiPgpConfig, error::UdiPgpResult, parser::UdiPgpQueryParser, simulations::response,
    sql_supplier::SqlSupplierType, Row,
};

pub mod query_handler;

#[derive(Debug, Clone)]
pub struct UdiPgpProcessor {
    query_parser: UdiPgpQueryParser,
    config: UdiPgpConfig,
    supplier: Arc<Mutex<SqlSupplierType>>,
}

impl UdiPgpProcessor {
    pub fn new(config: &UdiPgpConfig, supplier: SqlSupplierType) -> Self {
        UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
            config: config.clone(),
            supplier: Arc::new(Mutex::new(supplier)),
        }
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
            supplier: self.supplier.clone(),
        })
    }
}
