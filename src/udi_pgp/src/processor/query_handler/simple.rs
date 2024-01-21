use async_trait::async_trait;
use pgwire::{
    api::{
        query::SimpleQueryHandler,
        results::{QueryResponse, Response, Tag},
        ClientInfo,
    },
    error::PgWireResult,
};

use crate::{
    parser::UdiPgpQueryParser,
    processor::UdiPgpProcessor,
    simulations::{
        CLOSE_CURSOR, COMMIT_TRANSACTION, SET_DATE_STYLE, SET_EXTRA_FLOAT_DIGITS, SET_SEARCH_PATH,
        SET_TIME_ZONE, START_TRANSACTION,
    },
};

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

        let (schema, rows) = if statement.from_driver {
            match query {
                SET_SEARCH_PATH | SET_TIME_ZONE | SET_DATE_STYLE | SET_EXTRA_FLOAT_DIGITS => {
                    return Ok(vec![Response::Execution(Tag::new("SET"))])
                }
                CLOSE_CURSOR => return Ok(vec![Response::Execution(Tag::new("CLOSE"))]),
                START_TRANSACTION => return Ok(vec![Response::Execution(Tag::new("START"))]),
                COMMIT_TRANSACTION => return Ok(vec![Response::Execution(Tag::new("COMMIT"))]),
                _ => self.simulate_driver_responses(query)?,
            }
        } else {
            let mut supplier = self.supplier.lock().await;
            (
                supplier.schema(&statement).await?,
                supplier.execute(&statement).await?,
            )
        };

        let row_stream = self.encode_rows(schema.clone().into(), &rows);
        let response = Response::Query(QueryResponse::new(schema.into(), row_stream));

        Ok(vec![response])
    }
}
