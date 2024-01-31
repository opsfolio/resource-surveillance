use async_trait::async_trait;
use pgwire::{
    api::{query::SimpleQueryHandler, results::Response, ClientInfo},
    error::PgWireResult,
};
use tracing::debug;

use crate::{
    parser::{stmt::StmtType, UdiPgpQueryParser},
    processor::UdiPgpProcessor,
};

#[async_trait]
impl SimpleQueryHandler for UdiPgpProcessor {
    async fn do_query<'a, C>(
        &self,
        client: &mut C,
        query: &'a str,
    ) -> PgWireResult<Vec<Response<'a>>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let mut statement = UdiPgpQueryParser::parse(query, false)?;
        debug!("Executing query: {query}");
        debug!("Parsed statement: {:#?}", statement);

        match statement.stmt_type {
            StmtType::Config => self.handle_config(&statement).await,
            StmtType::Driver => self.handle_driver(query),
            StmtType::Supplier => self.handle_supplier(client, &mut statement).await,
            StmtType::Introspection => Ok(vec![]), // Add logic here if needed
        }
    }
}
