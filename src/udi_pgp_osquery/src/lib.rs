use async_trait::async_trait;
use pgwire::{api::results::FieldInfo, error::PgWireResult};
use udi_pgp::sql_supplier::SqlSupplier;

pub struct OsquerySupplier {}

#[async_trait]
impl SqlSupplier for OsquerySupplier {
    async fn schema(&mut self, query: String) -> PgWireResult<Vec<FieldInfo>> {
        Ok(vec![])
    }

    async fn execute(&mut self, query: String) -> PgWireResult<Vec<Vec<String>>> {
        Ok(vec![])
    }
}
