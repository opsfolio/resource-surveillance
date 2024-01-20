use async_trait::async_trait;
use pgwire::{
    api::{
        query::SimpleQueryHandler,
        results::{QueryResponse, Response},
        ClientInfo,
    },
    error::PgWireResult,
};

use crate::{parser::UdiPgpQueryParser, processor::UdiPgpProcessor};

#[async_trait]
impl SimpleQueryHandler for UdiPgpProcessor {
    async fn do_query<'a, C>(
        &self,
        _client: &mut C,
        query: &'a str,
    ) -> PgWireResult<Vec<Response<'a>>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        let statement = UdiPgpQueryParser::parse(query)?;
        println!("{:#?}", statement);
        let mut supplier = self.supplier.lock().await;
        let schema = supplier.schema(&statement).await?;
        let rows = supplier.execute(&statement).await?;
        let row_stream = self.encode_rows(schema.clone().into(), &rows);
        let response = Response::Query(QueryResponse::new(schema.into(), row_stream));
        Ok(vec![response])
    }
}
