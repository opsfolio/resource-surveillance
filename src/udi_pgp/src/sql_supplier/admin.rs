use std::{collections::HashMap, sync::Arc};

use pgwire::error::{ErrorInfo, PgWireError};
use tokio::sync::{Mutex, RwLock};

use crate::{
    config::{Supplier, UdiPgpConfig},
    error::{self, UdiPgpResult},
};

use super::{SqlSupplierMap, SqlSupplierType};

/// Factory for suppiers to register
#[derive(Debug, Clone, Default)]
pub struct UdiPgpSupplierFactory {
    /// Conatins the type of a supplier and how to create the supplier. For example, { osquery: generate_new }
    list: HashMap<String, fn(supplier: Supplier) -> UdiPgpResult<SqlSupplierType>>,
}

impl UdiPgpSupplierFactory {
    pub fn new() -> Self {
        UdiPgpSupplierFactory {
            list: HashMap::new(),
        }
    }

    pub fn register(
        &mut self,
        supplier_type: &str,
        generate_new_fn: fn(supplier: Supplier) -> UdiPgpResult<SqlSupplierType>,
    ) {
        self.list.insert(supplier_type.to_string(), generate_new_fn);
    }
}

/// The executive supplier to handle all other suppliers and introspection
#[derive(Debug, Clone)]
pub struct AdminSupplier {
    factory: Arc<RwLock<UdiPgpSupplierFactory>>,
    suppliers: Arc<RwLock<SqlSupplierMap>>,
}

impl AdminSupplier {
    pub fn new(suppliers: SqlSupplierMap, factory: UdiPgpSupplierFactory) -> Self {
        AdminSupplier {
            factory: Arc::new(RwLock::new(factory)),
            suppliers: Arc::new(RwLock::new(suppliers)),
        }
    }

    pub async fn current_suppliers(&self) -> usize {
        self.suppliers.read().await.len()
    }

    pub async fn supplier(&self, identifier: &str) -> UdiPgpResult<Arc<Mutex<SqlSupplierType>>> {
        let suppliers = self.suppliers.read().await;
        Ok(suppliers.get(identifier).cloned().ok_or_else(|| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "PROCESSOR".to_string(),
                format!("Supplier not found. Got: {}", identifier),
            )))
        })?)
    }

    // loop over the current suppliers. check the correspondings in config and then update them
    // for new suppliers:
    // - check the processor suppliers to get a template for that supplier type and create from that
    // - if no suppier type of that is present in the processor's suppliers list, throw an error
    // - the reeason for the error is that udi_pgp core knows nothing of how suppliers are created or what it needs, so, we can only
    //    infer new suppliers from templates(suppliers) that are already present, because those suppliers implement the SqlSupplier trait
    //     which provides a way to create a new one from itself.
    pub async fn update(&mut self, new_config: &UdiPgpConfig) -> UdiPgpResult<()> {
        let mut current_suppliers = self.suppliers.write().await;

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

        for (id, new_supplier_config) in &new_config.suppliers {
            if !current_suppliers.contains_key(id) {
                if let Some(supplier_fn) = self
                    .factory
                    .read()
                    .await
                    .list
                    .get(&new_supplier_config.supplier_type.to_string())
                {
                    let new_supplier = supplier_fn(new_supplier_config.clone())?;
                    current_suppliers.insert(id.clone(), Arc::new(Mutex::new(new_supplier)));
                } else {
                    return Err(error::UdiPgpError::PgWireError(PgWireError::UserError(
                        Box::new(ErrorInfo::new(
                            "FATAL".to_string(),
                            "PROCESSOR".to_string(),
                            format!(
                                "No supplier type found for supplier: {id} of type: {}.",
                                new_supplier_config.supplier_type
                            ),
                        )),
                    )));
                }
            }
        }

        Ok(())
    }
}
