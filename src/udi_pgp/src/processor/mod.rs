use std::{net::SocketAddr, path::PathBuf, pin::Pin, str::FromStr, sync::Arc};

use futures::{stream, Stream};
use pgwire::{
    api::{
        results::{DataRowEncoder, FieldInfo, QueryResponse, Response, Tag},
        MakeHandler,
    },
    error::{ErrorInfo, PgWireError, PgWireResult},
    messages::data::DataRow,
};
use sqlparser::ast::{self, Expr, Statement};
use tokio::sync::{mpsc, oneshot, RwLock};
use tracing::{debug, error};
use uuid::Uuid;

use crate::{
    config::UdiPgpConfig,
    error::{UdiPgpError, UdiPgpResult},
    health, metrics,
    parser::{stmt::UdiPgpStatment, UdiPgpQueryParser},
    simulations::{
        response, CLOSE_CURSOR, COMMIT_TRANSACTION, SET_DATE_STYLE, SET_EXTRA_FLOAT_DIGITS,
        SET_SEARCH_PATH, SET_TIME_ZONE, START_TRANSACTION,
    },
    sql_supplier::{
        admin::{AdminSupplier, UdiPgpSupplierFactory},
        SqlSupplierMap,
    },
    state::messages::Message,
    Row,
};

pub mod query_handler;

#[derive(Debug)]
pub struct UdiPgpProcessor {
    query_parser: UdiPgpQueryParser,
    config_tx: mpsc::Sender<Message>,
    exec_supplier: Arc<RwLock<AdminSupplier>>,
    health_shutdown: Arc<Option<oneshot::Sender<()>>>,
    metrics_shutdown: Arc<Option<oneshot::Sender<()>>>,
}

impl UdiPgpProcessor {
    pub async fn init(
        config_tx: mpsc::Sender<Message>,
        factory: UdiPgpSupplierFactory,
        suppliers: SqlSupplierMap,
    ) -> UdiPgpResult<Self> {
        let admin_supplier = AdminSupplier::new(suppliers, factory.clone());

        let mut processor = UdiPgpProcessor {
            query_parser: UdiPgpQueryParser::new(),
            config_tx,
            exec_supplier: Arc::new(RwLock::new(admin_supplier)),
            health_shutdown: Arc::new(None),
            metrics_shutdown: Arc::new(None),
        };
        processor.start_core_services().await?;
        Ok(processor)
    }

    async fn read_config(&self) -> UdiPgpResult<UdiPgpConfig> {
        let (response_tx, response_rx) = oneshot::channel();
        let read_state_msg = Message::ReadConfig(response_tx);
        self.config_tx
            .send(read_state_msg)
            .await
            .expect("Failed to send message");
        match response_rx.await {
            Ok(config) => {
                debug!("Latest Config: {:#?}", config);
                Ok(config)
            }
            Err(e) => {
                error!("{}", e);
                Err(UdiPgpError::ConfigError(format!(
                    "Failed to read configuration: {}",
                    e
                )))
            }
        }
    }

    async fn start_core_services(&mut self) -> UdiPgpResult<()> {
        let (health_tx, health_rx) = oneshot::channel::<()>();
        let (metrics_tx, metrics_rx) = oneshot::channel::<()>();

        // Store shutdown senders to trigger shutdown later when I need them
        self.health_shutdown = Some(health_tx).into();
        self.metrics_shutdown = Some(metrics_tx).into();

        let config = self.read_config().await?;

        let health_addr = { config.health };

        tokio::spawn(async move {
            if let Err(e) = UdiPgpProcessor::start_health_server(health_addr, health_rx).await {
                error!("Failed to start health server: {}", e);
            }
        });

        let metrics_addr = { config.metrics };
        tokio::spawn(async move {
            if let Err(e) = UdiPgpProcessor::start_metrics_server(metrics_addr, metrics_rx).await {
                error!("Failed to start metrics server: {}", e);
            }
        });

        Ok(())
    }

    async fn start_health_server(
        address: Option<SocketAddr>,
        rx: oneshot::Receiver<()>,
    ) -> UdiPgpResult<()> {
        if let Some(addr) = address {
            let _ = health::start(addr, rx).await;
        }
        Ok(())
    }

    async fn start_metrics_server(
        address: Option<SocketAddr>,
        rx: oneshot::Receiver<()>,
    ) -> UdiPgpResult<()> {
        if let Some(addr) = address {
            let _ = metrics::start(addr, rx).await;
        }
        Ok(())
    }

    pub(crate) fn extract_supplier_and_database(
        param: Option<&str>,
    ) -> PgWireResult<(String, Option<String>)> {
        let db = param.ok_or_else(|| {
            error!("Cannot find database parameter");
            PgWireError::UserError(Box::new(ErrorInfo::new(
                "FATAL".to_string(),
                "PROCESSOR".to_string(),
                "Cannot find database parameter".to_string(),
            )))
        })?;

        let parts: Vec<&str> = db.split(':').collect();

        let supplier = parts
            .first()
            .ok_or_else(|| {
                PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "PROCESSOR".to_string(),
                    "Supplier is absent".to_string(),
                )))
            })?
            .to_string();

        let identifier = parts.get(1).map(|s| s.to_string());

        Ok((supplier, identifier))
    }

    pub fn encode_rows(
        &self,
        schema: Arc<Vec<FieldInfo>>,
        rows: &[Vec<Row>],
    ) -> Pin<Box<dyn Stream<Item = PgWireResult<DataRow>> + Send + Sync>> {
        debug!("encoding rows");

        let mut results = Vec::new();
        let ncols = schema.len();

        rows.iter().for_each(|row| {
            let mut encoder = DataRowEncoder::new(schema.clone());
            for idx in 0..ncols {
                let data = &row.get(idx).unwrap().value;
                encoder.encode_field(&data).unwrap();
            }

            results.push(encoder.finish());
        });

        debug!("encoded rows successfully");
        Box::pin(stream::iter(results))
    }

    pub async fn handle_config<'a>(
        &self,
        statement: &UdiPgpStatment,
        query_id: &Uuid,
    ) -> PgWireResult<Vec<Response<'a>>> {
        if let Err(err) = self.update(statement, query_id).await {
            error!("{}", err);
            return Err(err);
        };
        Ok(vec![Response::Execution(Tag::new("UDI-PGP CONFIG SET"))])
    }

    pub fn handle_driver<'a>(&self, query: &'a str) -> PgWireResult<Vec<Response<'a>>> {
        match query {
            SET_SEARCH_PATH | SET_TIME_ZONE | SET_DATE_STYLE | SET_EXTRA_FLOAT_DIGITS => {
                Ok(vec![Response::Execution(Tag::new("SET"))])
            }
            CLOSE_CURSOR => Ok(vec![Response::Execution(Tag::new("CLOSE"))]),
            START_TRANSACTION => Ok(vec![Response::Execution(Tag::new("START"))]),
            COMMIT_TRANSACTION => Ok(vec![Response::Execution(Tag::new("COMMIT"))]),
            _ => {
                let (schema, rows) = self.simulate_driver_responses(query)?;

                let row_stream = self.encode_rows(schema.clone().into(), &rows);
                let response = Response::Query(QueryResponse::new(schema.into(), row_stream));

                Ok(vec![response])
            }
        }
    }

    fn simulate_driver_responses(
        &self,
        query: &str,
    ) -> UdiPgpResult<(Vec<FieldInfo>, Vec<Vec<Row>>)> {
        let (schema, rows) = response::driver_queries_response(query)?;
        let rows = vec![rows
            .into_iter()
            .map(|r| Row::from_str(r).unwrap())
            .collect::<Vec<_>>()];
        Ok((schema, rows))
    }

    async fn send_set_record_msg(
        &self,
        file: PathBuf,
        config_str: String,
        query_id: &Uuid,
        query: &str,
    ) -> PgWireResult<()> {
        let create_set_record = Message::CreateConfigQueryLog {
            query_id: query_id.to_string(),
            query_text: query.to_string(),
            generated_ncl: config_str,
            diagnostics_file: file.to_str().unwrap().to_string(),
        };
        self.config_tx
            .send(create_set_record)
            .await
            .map_err(|err| {
                error!("Failed to send message to create SET recor. Error: {}", err);
                PgWireError::ApiError(Box::new(err))
            })?;
        Ok(())
    }

    async fn handle_set_config(
        &self,
        name: &str,
        config_str: String,
        query_id: &Uuid,
        query: &str,
    ) -> PgWireResult<()> {
        let diagnostics_file = match name {
            "udi_pgp_serve_ncl_supplier" => {
                let diagnostics_file_path = UdiPgpConfig::diagnostics(&config_str, false)?;
                let (id, new_supplier) = match UdiPgpConfig::try_supplier_from_diagnostics(
                    &config_str,
                    &diagnostics_file_path,
                ) {
                    Ok(res) => res,
                    Err(err) => {
                        self.send_set_record_msg(
                            diagnostics_file_path,
                            config_str,
                            query_id,
                            query,
                        )
                        .await?;
                        return Err(err.into());
                    }
                };

                let update_supplier_msg = Message::InsertSupplier(id, new_supplier.clone());

                self.config_tx
                    .send(update_supplier_msg)
                    .await
                    .map_err(|err| {
                        error!(
                            "Failed to send message to insert supplier: {:#?}. Error: {}",
                            new_supplier, err
                        );
                        PgWireError::ApiError(Box::new(err))
                    })?;
                diagnostics_file_path
            }
            "udi_pgp_serve_ncl_core" => {
                let diagnostics_file_path = UdiPgpConfig::diagnostics(&config_str, true)?;

                let core = match UdiPgpConfig::try_config_from_diagnostics(&diagnostics_file_path) {
                    Ok(config) => config,
                    Err(err) => {
                        self.send_set_record_msg(
                            diagnostics_file_path,
                            config_str,
                            query_id,
                            query,
                        )
                        .await?;
                        return Err(err.into());
                    }
                };
                // TODO use chamged features to open ports
                let update_core_msg = Message::UpdateCore(core.metrics, core.health);
                self.config_tx.send(update_core_msg).await.map_err(|err| {
                    error!("Failed to send message to update core. {}", err);
                    PgWireError::ApiError(Box::new(err))
                })?;
                diagnostics_file_path
            }
            other => {
                return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                    "FATAL".to_string(),
                    "PROCESSOR".to_string(),
                    format!("Config query not supported, got: {:?}", other),
                ))))
            }
        };

        self.send_set_record_msg(diagnostics_file, config_str, query_id, query)
            .await?;

        Ok(())
    }

    async fn update(&self, stmt: &UdiPgpStatment, query_id: &Uuid) -> PgWireResult<()> {
        let ast = &stmt.stmt;

        match ast {
            Statement::SetVariable {
                variable, value, ..
            } => {
                let name = variable
                    .0
                    .first()
                    .ok_or_else(|| {
                        PgWireError::UserError(Box::new(ErrorInfo::new(
                            "WARNING".to_string(),
                            "PARSER".to_string(),
                            "Variable name is missing".to_string(),
                        )))
                    })?
                    .value
                    .as_str();

                if !name.starts_with("udi_pgp_serve_") {
                    return Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                        "WARNING".to_string(),
                        "PARSER".to_string(),
                        format!(
                            "Expected variable to start with 'udi_pgp_serve_', got: {}",
                            name
                        ),
                    ))));
                }

                let config_str =
                    self.extract_single_quoted_string(value.first().ok_or_else(|| {
                        PgWireError::UserError(Box::new(ErrorInfo::new(
                            "WARNING".to_string(),
                            "PARSER".to_string(),
                            "Value is missing".to_string(),
                        )))
                    })?)?;

                self.handle_set_config(name, config_str, query_id, &stmt.query)
                    .await?;

                let mut exec_supplier = self.exec_supplier.write().await;
                // config has been updated above, so to get the latest update
                let config = self.read_config().await?;
                exec_supplier.update(&config).await?;

                Ok(())
            }
            _ => Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "WARNING".to_string(),
                "PARSER".to_string(),
                format!("Expected SET statement, got: {:?}", ast),
            )))),
        }
    }

    fn extract_single_quoted_string(&self, expr: &Expr) -> Result<String, PgWireError> {
        if let Expr::Value(ast::Value::SingleQuotedString(s)) = expr {
            Ok(s.to_string())
        } else {
            Err(PgWireError::UserError(Box::new(ErrorInfo::new(
                "WARNING".to_string(),
                "PARSER".to_string(),
                format!("Expected a single quoted string, got: {:?}", expr),
            ))))
        }
    }
}

impl MakeHandler for UdiPgpProcessor {
    type Handler = Arc<UdiPgpProcessor>;

    fn make(&self) -> Self::Handler {
        Arc::new(UdiPgpProcessor {
            config_tx: self.config_tx.clone(),
            query_parser: self.query_parser.clone(),
            exec_supplier: self.exec_supplier.clone(),
            health_shutdown: self.health_shutdown.clone(),
            metrics_shutdown: self.metrics_shutdown.clone(),
        })
    }
}
