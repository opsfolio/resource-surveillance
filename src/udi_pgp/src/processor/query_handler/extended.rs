use std::sync::Arc;

use async_trait::async_trait;
use pgwire::{
    api::{
        portal::Portal,
        query::{ExtendedQueryHandler, StatementOrPortal},
        results::{DescribeResponse, Response},
        ClientInfo,
    },
    error::PgWireResult,
};

use crate::{parser::UdiPgpQueryParser, processor::UdiPgpProcessor};

#[async_trait]
impl ExtendedQueryHandler for UdiPgpProcessor {
    type Statement = String;
    type QueryParser = UdiPgpQueryParser;

    fn query_parser(&self) -> Arc<Self::QueryParser> {
        self.query_parser.clone().into()
    }

    async fn do_query<'a, C>(
        &self,
        _client: &mut C,
        portal: &'a Portal<Self::Statement>,
        _max_rows: usize,
    ) -> PgWireResult<Response<'a>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        Ok(Response::EmptyQuery)
    }

    async fn do_describe<C>(
        &self,
        _client: &mut C,
        target: StatementOrPortal<'_, Self::Statement>,
    ) -> PgWireResult<DescribeResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        Ok(DescribeResponse::new(None, vec![]))
    }
}
