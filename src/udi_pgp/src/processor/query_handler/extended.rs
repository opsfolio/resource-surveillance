use std::sync::Arc;

use async_trait::async_trait;
use pgwire::{
    api::{
        portal::Portal, query::ExtendedQueryHandler, results::{DescribePortalResponse, DescribeStatementResponse, Response}, stmt::StoredStatement, ClientInfo
    },
    error::PgWireResult,
};

use crate::{
    parser::{stmt::UdiPgpStatment, UdiPgpQueryParser},
    processor::UdiPgpProcessor,
};

#[async_trait]
impl ExtendedQueryHandler for UdiPgpProcessor {
    type Statement = UdiPgpStatment;
    type QueryParser = UdiPgpQueryParser;

    fn query_parser(&self) -> Arc<Self::QueryParser> {
        self.query_parser.clone().into()
    }

    async fn do_query<'a, C>(
        &self,
        _client: &mut C,
        _portal: &'a Portal<Self::Statement>,
        _max_rows: usize,
    ) -> PgWireResult<Response<'a>>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        Ok(Response::EmptyQuery)
    }

    async fn do_describe_statement<C>(
        &self,
        _client: &mut C,
        _target: &StoredStatement<Self::Statement>,
    ) -> PgWireResult<DescribeStatementResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        Ok(DescribeStatementResponse::new(vec![], vec![]))
    }

    async fn do_describe_portal<C>(
        &self,
        _client: &mut C,
        _portal: &Portal<Self::Statement>,
    ) -> PgWireResult<DescribePortalResponse>
    where
        C: ClientInfo + Unpin + Send + Sync,
    {
        todo!()
    }
}
