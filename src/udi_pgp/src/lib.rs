use std::sync::OnceLock;
use std::{fmt::Display, str::FromStr, sync::Arc};

use config::UdiPgpConfig;
use derive_new::new;
use error::UdiPgpError;
use pgwire::{api::MakeHandler, tokio::process_socket};
use serde::Deserialize;
use sql_supplier::SqlSupplierMap;
use startup::{UdiPgpParameters, UdiPgpStartupHandler};
use tokio::sync::{mpsc, Mutex};
use tokio::{net::TcpListener, signal, sync::oneshot};
use tracing::debug;
use tracing::{error, info};

use crate::processor::UdiPgpProcessor;
use crate::startup::UdiPgpAuthSource;
use crate::state::StateManager;

mod health;
mod introspection;
mod metrics;
mod observability;
mod processor;
mod simulations;
mod startup;
mod state;

pub mod auth;
pub mod config;
pub mod error;
pub mod parser;
pub mod sql_supplier;
pub mod ssh;

pub use pgwire::api::results::FieldFormat;
pub use pgwire::api::results::FieldInfo;
pub use pgwire::api::Type;

use sql_supplier::admin::UdiPgpSupplierFactory;

static INSTANCE: OnceLock<Mutex<UdiPgpSupplierFactory>> = OnceLock::new();

#[allow(non_snake_case)]
pub fn FACTORY() -> &'static Mutex<UdiPgpSupplierFactory> {
    INSTANCE.get_or_init(|| Mutex::new(UdiPgpSupplierFactory::new()))
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UdiPgpModes {
    Local,
    Remote,
}

impl Display for UdiPgpModes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UdiPgpModes::Local => f.write_str("execution on local machine"),
            UdiPgpModes::Remote => f.write_str("execution on remote machine"),
        }
    }
}

#[derive(Debug, Clone, new)]
pub struct Row {
    pub value: String,
}

impl FromStr for Row {
    type Err = UdiPgpError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Row {
            value: s.to_string(),
        })
    }
}

impl From<String> for Row {
    fn from(value: String) -> Self {
        Row { value }
    }
}

fn spawn_shutdown_handler() -> oneshot::Receiver<()> {
    let (tx, rx) = oneshot::channel();
    // TODO check connected instances before shutting down
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(()) => {
                info!("shutdown triggered");
                // Shutdown!
                let _ = tx.send(());
            }
            Err(err) => {
                error!(%err, "unable to listen for shutdown signal");
            }
        }
    });
    rx
}

pub async fn run(config: &UdiPgpConfig, suppliers: SqlSupplierMap) -> anyhow::Result<()> {
    debug!("Starting the pgp server with: {:#?}", config);

    let (tx, rx) = mpsc::channel(32);

    {
        let mut state_manager =
            StateManager::init(config)?;
        tokio::spawn(async move {
            state_manager.handle(rx).await;
        });
    }

    observability::init(&tx, config.verbose)?;

    let authenticator = Arc::new(UdiPgpStartupHandler::new(
        UdiPgpAuthSource::new(tx.clone()),
        UdiPgpParameters::new(),
        tx.clone(),
    ));

    let factory = FACTORY().lock().await;
    let processor = UdiPgpProcessor::init(tx.clone(), factory.clone(), suppliers).await?;

    let mut rx = spawn_shutdown_handler();
    let listener = TcpListener::bind(config.addr()).await?;

    info!("UDI PGP SQLD listening on {}", config.addr());
    loop {
        tokio::select! {
            _ = &mut rx => {
                info!("shutting down");
                return Ok(())
            }

            incoming_socket = listener.accept() => {
                let (connection, _) = incoming_socket?;
                let authenticator_ref = authenticator.clone();
                let processor_ref = processor.make();
                tokio::spawn(async move {
                    process_socket(
                        connection,
                        None,
                        authenticator_ref,
                        processor_ref.clone(),
                        processor_ref,
                    )
                    .await
                });
            }
        }
    }
}
