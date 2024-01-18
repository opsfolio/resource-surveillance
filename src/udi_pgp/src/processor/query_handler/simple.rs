use async_trait::async_trait;
use pgwire::{
    api::{query::SimpleQueryHandler, results::Response, ClientInfo},
    error::PgWireResult,
};

use crate::processor::UdiPgpProcessor;

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
        println!("{query}");
        Ok(vec![Response::EmptyQuery])
    }
}
