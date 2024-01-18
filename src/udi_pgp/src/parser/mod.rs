use async_trait::async_trait;
use derive_new::new;
use pgwire::{
    api::{stmt::QueryParser, Type},
    error::PgWireResult,
};

#[derive(new, Debug, Default, Clone)]
// If I were to add datafusion, this is where it would come in.
// Notes: when it is time for udi specific queries, the queries could be deconstructed here
pub struct UdiPgpQueryParser;

#[async_trait]
impl QueryParser for UdiPgpQueryParser {
    type Statement = String;

    async fn parse_sql(&self, sql: &str, _types: &[Type]) -> PgWireResult<Self::Statement> {
        Ok(sql.to_owned())
    }
}
