use async_trait::async_trait;
use pgwire::{
    api::{
        query::SimpleQueryHandler,
        results::{QueryResponse, Response},
        ClientInfo,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
};
use tracing::{debug, debug_span, info, info_span, Instrument};
use uuid::Uuid;

use crate::{
    introspection::Introspection,
    parser::{
        stmt::{StmtType, UdiPgpStatment},
        UdiPgpQueryParser,
    },
    processor::UdiPgpProcessor,
};

impl UdiPgpProcessor {
    async fn handle_supplier<'a, C: ClientInfo + Unpin + Send + Sync>(
        &self,
        client: &mut C,
        statement: &mut UdiPgpStatment,
        session_id: &Uuid,
    ) -> PgWireResult<Vec<Response<'a>>> {
        let metadata = client.metadata();
        let (supplier_id, _) =
            Self::extract_supplier_and_database(metadata.get("database").map(|x| x.as_str()))?;

        let exec_supplier = self.exec_supplier.read().await;
        let supplier = exec_supplier.supplier(&supplier_id).await?;
        let mut supplier = supplier.lock().await;
        supplier.add_session_id(*session_id)?;

        info!("Supplier: {supplier_id} currently in use.");
        let (schema, rows) = (
            supplier.schema(statement).await?,
            supplier.execute(statement).await?,
        );

        let row_stream = self.encode_rows(schema.clone().into(), &rows);
        let response = Response::Query(QueryResponse::new(schema.into(), row_stream));

        Ok(vec![response])
    }

    async fn handle_introspection<'a>(
        &self,
        stmt: &UdiPgpStatment,
        session_id: &Uuid,
    ) -> PgWireResult<Vec<Response<'a>>> {
        let mut introspection =
            Introspection::new(stmt, self.config_tx.clone()).map_err(|err| {
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "INTROSPECTION".to_string(),
                    err.to_string(),
                )))
            })?;

        let (schema, rows) = introspection.handle(session_id).await.map_err(|err| {
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
        let query_id = Uuid::new_v4();

        let span = if config.verbose {
            debug_span!("simple query handler", query_text = query, query_id = ?query_id)
        } else {
            info_span!("simple query handler", query_text = query, query_id = ?query_id)
        };

        async {
            let mut statement = UdiPgpQueryParser::parse(query, false)?;
            debug!("Executing query: {query}");
            debug!("Parsed statement: {:#?}", statement);

            Ok(match statement.stmt_type {
                StmtType::Config => self.handle_config(&statement, &query_id).await?,
                StmtType::Driver => self.handle_driver(query)?,
                StmtType::Supplier => {
                    self.handle_supplier(client, &mut statement, &query_id)
                        .await?
                }
                StmtType::Introspection => self.handle_introspection(&statement, &query_id).await?,
            })
        }
        .instrument(span)
        .await
    }
}
