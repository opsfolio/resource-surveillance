use std::{fmt::Display, sync::Arc};

use config::UdiPgpConfig;
use derive_new::new;
use pgwire::{api::MakeHandler, tokio::process_socket};
use sql_supplier::SqlSupplierType;
use startup::{UdiPgpParameters, UdiPgpStartupHandler};
use tokio::{net::TcpListener, signal, sync::oneshot};
use tracing::{error, info};

use crate::processor::UdiPgpProcessor;

mod processor;
mod startup;

pub mod parser;
pub mod auth;
pub mod config;
pub mod sql_supplier;
pub mod error;
pub use pgwire::api::results::FieldInfo;

#[derive(Debug, Clone)]
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
    pub value: String
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

pub async fn run(config: &UdiPgpConfig, supplier: SqlSupplierType) -> anyhow::Result<()> {
    let authenticator = Arc::new(UdiPgpStartupHandler::new(
        config.auth().clone(),
        UdiPgpParameters::new(),
    ));
    let processor = UdiPgpProcessor::new(config, supplier);
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
