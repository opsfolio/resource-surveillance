use async_trait::async_trait;
use pgwire::{api::results::FieldInfo, error::PgWireResult};

#[async_trait]
pub trait SqlSupplier {
    async fn schema(&mut self, query: String) -> PgWireResult<Vec<FieldInfo>>;

    async fn execute(&mut self, query: String) -> PgWireResult<Vec<Vec<String>>>;
}
