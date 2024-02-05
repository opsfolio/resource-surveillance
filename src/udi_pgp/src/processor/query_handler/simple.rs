use async_trait::async_trait;
use pgwire::{
    api::{
        query::SimpleQueryHandler,
        results::{QueryResponse, Response},
        ClientInfo,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
};
use tracing::{debug, debug_span, info_span, Instrument};

use crate::{
    introspection::Introspection,
    parser::{
        stmt::{StmtType, UdiPgpStatment},
        UdiPgpQueryParser,
    },
    processor::UdiPgpProcessor,
};

impl UdiPgpProcessor {
    async fn handle_introspection<'a>(
        &self,
        stmt: &UdiPgpStatment,
    ) -> PgWireResult<Vec<Response<'a>>> {
        let mut introspection =
            Introspection::new(stmt, self.config_tx.clone()).map_err(|err| {
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "INTROSPECTION".to_string(),
                    err.to_string(),
                )))
            })?;

        let (schema, rows) = introspection.handle().await.map_err(|err| {
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "INTROSPECTION".to_string(),
                err.to_string(),
            )))
        })?;

        let row_stream = self.encode_rows(schema.clone().into(), &rows);
        let response = Response::Query(QueryResponse::new(schema.into(), row_stream));

        Ok(vec![response])
    }
}

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
        let config = self.read_config().await?;

        let span = if config.verbose {
            debug_span!("simple query handler", query)
        } else {
            info_span!("simple query handler", query)
        };

        async {
            let mut statement = UdiPgpQueryParser::parse(query, false)?;
            debug!("Executing query: {query}");
            debug!("Parsed statement: {:#?}", statement);

            Ok(match statement.stmt_type {
                StmtType::Config => self.handle_config(&statement).await?,
                StmtType::Driver => self.handle_driver(query)?,
                StmtType::Supplier => self.handle_supplier(client, &mut statement).await?,
                StmtType::Introspection => self.handle_introspection(&statement).await?,
            })
        }
        .instrument(span)
        .await
    }
}
