use std::{collections::HashMap, pin::Pin, str::FromStr, sync::Arc};

use futures::{stream, Stream};
use pgwire::{
    api::{
        results::{DataRowEncoder, FieldInfo},
        MakeHandler,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
    messages::data::DataRow,
};
use sqlparser::ast::{self, Expr, Statement};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error};

use crate::{
    config::UdiPgpConfig,
    error::UdiPgpResult,
    parser::{stmt::UdiPgpStatment, UdiPgpQueryParser},
    simulations::response,
    sql_supplier::{SqlSupplierMap, SqlSupplierType},
    Row,
};

pub mod query_handler;

#[derive(Debug, Clone)]
pub struct UdiPgpProcessor {
    query_parser: UdiPgpQueryParser,
    config: Arc<RwLock<UdiPgpConfig>>,
    suppliers: Arc<RwLock<SqlSupplierMap>>,
}

impl UdiPgpProcessor {
    pub fn new(config: Arc<RwLock<UdiPgpConfig>>, suppliers: SqlSupplierMap) -> Self {
        UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
            config,
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

    // pub async fn start_metrics(&self, port: u16, former_port: Option<u16>) -> UdiPgpResult<()> {
    //     Ok(())
    // }

    pub(crate) fn extract_supplier_and_database(
        param: Option<&str>,
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

        let supplier = parts
            .first()
            .ok_or_else(|| {
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "PROCESSOR".to_string(),
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

    async fn update(
        &self,
        config: &mut UdiPgpConfig,
        suppliers: &mut SqlSupplierMap,
        stmt: &UdiPgpStatment,
    ) -> PgWireResult<()> {
        let ast = &stmt.stmt;

        match ast {
            Statement::SetVariable {
                variable, value, ..
            } => {
                let name = variable
                    .0
                    .first()
                    .ok_or_else(|| {
                        PgWireError::UserError(Box::new(ErrorInfo::new(
                            "WARNING".to_string(),
                            "PARSER".to_string(),
                            "Variable name is missing".to_string(),
                        )))
                    })?
                    .value
                    .as_str();

                if !name.starts_with("udi_pgp_serve_") {
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "WARNING".to_string(),
                        "PARSER".to_string(),
                        format!(
                            "Expected variable to start with 'udi_pgp_serve_', got: {}",
                            name
                        ),
                    ))));
                }

                let config_str =
                    self.extract_single_quoted_string(value.first().ok_or_else(|| {
                        PgWireError::UserError(Box::new(ErrorInfo::new(
                            "WARNING".to_string(),
                            "PARSER".to_string(),
                            "Value is missing".to_string(),
                        )))
                    })?)?;

                let mut new_config = config.clone();
                match name {
                    "udi_pgp_serve_ncl_supplier" => {
                        let (id, new_supplier) =
                            UdiPgpConfig::try_config_from_ncl_serve_supplier(&config_str)?;
                        new_config.suppliers.insert(id, new_supplier);
                    }
                    "udi_pgp_serve_ncl_core" => {
                        let core = UdiPgpConfig::try_from_ncl_string(&config_str)?;
                        // TODO use chamged features to open ports
                        new_config.health = core.health;
                        new_config.metrics = core.metrics;
                    }
                    _ => {}
                };

                self.refresh(config, suppliers, &new_config).await
            }
            _ => Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "WARNING".to_string(),
                "PARSER".to_string(),
                format!("Expected SET statement, got: {:?}", ast),
            )))),
        }
    }

    fn extract_single_quoted_string(&self, expr: &Expr) -> Result<String, PgWireError> {
        if let Expr::Value(ast::Value::SingleQuotedString(s)) = expr {
            Ok(s.to_string())
        } else {
            Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "WARNING".to_string(),
                "PARSER".to_string(),
                format!("Expected a single quoted string, got: {:?}", expr),
            ))))
        }
    }

    // loop over the current suppliers. check the correspondings in config and then update them
    // for new suppliers:
    // - check the processor suppliers to get a template for that supplier type and create from that
    // - if no suppier type of that is present in the processor's suppliers list, throw an error
    // - the reeason for the error is that udi_pgp core knows nothing of how suppliers are created or what it needs, so, we can only
    //    infer new suppliers from templates(suppliers) that are already present, because those suppliers implement the SqlSupplier trait
    //     which provides a way to create a new one from itself.
    async fn refresh(
        &self,
        config: &mut UdiPgpConfig,
        current_suppliers: &mut SqlSupplierMap,
        new_config: &UdiPgpConfig,
    ) -> PgWireResult<()> {
        //TODO: updating and removing suppliers as per the new config could have been accomplished using
        // the retain method on current_suplliers hashmap, but since each supplier is in an async Mutex
        // acquiring the lock requires ".await" which needs to be in the cintext of an async function
        // but the retain closure can't be asynchronous. Try this again in future Rust releases.

        // get suppliers to remove
        let suppliers_to_remove: Vec<String> = current_suppliers
            .keys()
            .filter(|id| !new_config.suppliers.contains_key(*id))
            .cloned()
            .collect();

        // Remove the suppliers not present in new config
        for id in suppliers_to_remove {
            current_suppliers.remove(&id);
        }

        for (id, supplier) in current_suppliers.iter_mut() {
            if let Some(new_supplier_config) = new_config.suppliers.get(id) {
                // Update the supplier if it exists in new_config
                // TODO implement comparing. Compare if they are the same first before updating, To accomplish that:
                // 1. add a .to_supplier() method on SqlSupplier trait
                // 2. Implement the Eq and PartialEq trait for config::Supplier
                debug!(
                    "Updating supplier: {id} to new state: {:#?}",
                    new_supplier_config
                );
                let mut supplier_lock = supplier.lock().await;
                let _ = supplier_lock.update(new_supplier_config.clone());
            }
        }

        //get templates for new supplier types beforehand
        let mut templates = HashMap::new();
        for supplier in current_suppliers.values() {
            let supplier_lock = supplier.lock().await;
            templates.insert(supplier_lock.supplier_type(), supplier.clone());
        }

        // Add new suppliers
        for (id, new_supplier_config) in &new_config.suppliers {
            if !current_suppliers.contains_key(id) {
                if let Some(template_supplier) = templates.get(&new_supplier_config.supplier_type) {
                    let new_supplier = template_supplier
                        .lock()
                        .await
                        .generate_new(new_supplier_config.clone())?;
                    current_suppliers.insert(id.clone(), Arc::new(Mutex::new(new_supplier)));
                } else {
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "FATAL".to_string(),
                        "PROCESSOR".to_string(),
                        format!(
                            "No template supplier found for supplier: {id} of type: {}",
                            new_supplier_config.supplier_type
                        ),
                    ))));
                }
            }
        }

        *config = new_config.clone();
        Ok(())
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
