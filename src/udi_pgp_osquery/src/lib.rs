use async_trait::async_trait;
use udi_pgp::{
    error::UdiPgpResult, parser::stmt::UdiPgpStatment, sql_supplier::SqlSupplier, FieldInfo, Row,
    UdiPgpModes,
};

#[derive(Debug, Clone)]
pub struct OsquerySupplier {
    pub mode: UdiPgpModes,
}

impl OsquerySupplier {
    pub fn new(mode: UdiPgpModes) -> Self {
        OsquerySupplier { mode }
    }
}

#[async_trait]
impl SqlSupplier for OsquerySupplier {
    fn name(&self) -> &str {
        "osquery"
    }

    async fn schema(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<FieldInfo>> {
        Ok(vec![])
    }

    async fn execute(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<Vec<Row>>> {
        Ok(vec![])
    }
}
