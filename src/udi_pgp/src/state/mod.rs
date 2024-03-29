//! # UDI-PGP State Manager
//!
//! This module provides an asynchronous state manager for UDI-PGP. Seeing that multiple parts of UDI-PGP need
//! access to configuration and the database like the authenticator and processor. This is used to keep the updates in sync.
//! Since there'll be a lot writes to the config and comparable number of reads, using channels to handle the configuration
//! state makes sense as there are no expensive computations to be done, just setting and removing values
//!
//! ## Overview
//!
//! The configuration manager is built around a Tokio async mpsc channel, allowing various parts
//! of the application to send configuration update requests to a central manager task. This task
//! maintains the current state and handles updates and queries asynchronously.
//!
//! ## Usage
//!
//! The main components are:
//! - `Message`: Enum for representing different types of configuration messages.
//!
//! Example of sending a configuration update:
//!
//! ```no_run
//! # use tokio::sync::mpsc;
//! # use tokio::task;
//! # use tokio::time::Duration;
//! # enum ConfigMessage { UpdateConfig /* ... */, ReadConfig /* ... */ }
//! # async fn example(tx: mpsc::Sender<ConfigMessage>) {
//! tx.send(ConfigMessage::UpdateConfig(/* ... */)).await.unwrap();
//! # }
//! ```
//!
//! For more details on each component, see the respective function documentation.

use std::{collections::HashMap, fs, sync::Arc};

use rusqlite::{Connection, Result as RusqliteResult, ToSql};
use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, Level};
use uuid::Uuid;

use crate::{config::UdiPgpConfig, observability::QueryLogEntryMap};
use common::{execute_sql, execute_sql_batch};

use self::messages::{Message, UpdateLogEntry};

mod database;
pub mod messages;

execute_sql_batch!(admin_ddl, include_str!("../admin.sql"));
execute_sql!(
    insert_udi_pgp_config,
    "INSERT INTO udi_pgp_config (udi_pgp_config_id, addr, health, metrics, config_ncl, admin_db_path, surveilr_version, governance, created_at, created_by) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7,?8, CURRENT_TIMESTAMP, 'UNKNOWN')",
    udi_pgp_config_id: String,
    addr: String,
    health: Option<String>,
    metrics: Option<String>,
    config_ncl: String,
    admin_db_path: String,
    surveilr_version: String,
    governance: Option<String>
);

/// State Manager
pub struct StateManager {
    config: Arc<Mutex<UdiPgpConfig>>,
    log_entries: Arc<Mutex<QueryLogEntryMap>>,
    conn: Connection,
}

impl StateManager {
    /// Initialize the state manager, load the databse with tables and insert the core config
    pub fn init(config: &UdiPgpConfig) -> anyhow::Result<Self> {
        let connection = Connection::open(&config.admin_state_fs_path)?;

        admin_ddl(&connection)?;
        let admindb_path = config.admin_state_fs_path.to_str().unwrap();

        insert_udi_pgp_config(
            &connection,
            Uuid::new_v4().to_string(),
            config.addr().to_string(),
            config.health.map(|s| s.to_string()),
            config.metrics.map(|s| s.to_string()),
            "".to_string(),
            admindb_path.to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
            None,
        )?;

        Ok(StateManager {
            config: Arc::new(Mutex::new(config.clone())),
            log_entries: Arc::new(Mutex::new(HashMap::new())),
            conn: connection,
        })
    }

    pub async fn handle(&mut self, mut rx: mpsc::Receiver<Message>) {
        let shared_config = &self.config;
        let log_entries = &self.log_entries;

        while let Some(message) = rx.recv().await {
            match message {
                Message::ReadConfig(response_tx) => {
                    debug!("Attempting to acquire lock to read config");
                    let state = shared_config.lock().await;
                    debug!("Read config lock acquired");
                    let state_info = state.clone();

                    if response_tx.send(state_info).is_err() {
                        error!("Failed to send config back to sender");
                    }
                    debug!("Read config lock released");
                }
                Message::UpdateCore(metrics, health) => {
                    debug!(
                        "Updating Core Config with metrics and health addresses: {:#?}, {:#?}",
                        metrics, health
                    );
                    let mut config = shared_config.lock().await;
                    config.metrics = metrics;
                    config.health = health;
                    debug!("Updated Core Config Successfully");
                    self.update_core(&config);
                }
                Message::InsertSupplier(id, supplier) => {
                    debug!(
                        "Updating suppliers with supplier_id: {id} and supplier: {:#?}",
                        supplier
                    );
                    let mut config = shared_config.lock().await;
                    config.suppliers.insert(id, supplier);
                    debug!("Supplier updated successfully",);
                    self.update_suppliers(&config);
                }
                Message::ReadLogEntries(response_tx) => {
                    debug!("Attempting to acquire lock to read log entries");
                    let state = log_entries.lock().await;
                    debug!("Read log entries lock acquired");
                    let state_info = state.clone();

                    if response_tx.send(state_info).is_err() {
                        error!("Failed to send log entries back to sender");
                    }
                    debug!("Read log entries lock released");
                }
                Message::AddLogEntry { log, span_id } => {
                    let mut logs = log_entries.lock().await;
                    logs.entry(span_id).or_insert_with(|| log);
                }
                Message::UpdateLogEntry { span_id, msg } => {
                    let mut logs = log_entries.lock().await;
                    logs.entry(span_id.clone()).and_modify(|e| match msg {
                        UpdateLogEntry::Event(event, level) => {
                            e.elaboration.events.push(event.clone());
                            if level == Level::ERROR {
                                e.exec_msg.push(event);
                            }
                        }
                        UpdateLogEntry::EndTime(t) => {
                            e.exec_finish_at = Some(t);
                            self.insert_query_log(e);
                            self.update_udi_pgp_set_record(e);
                        }
                        UpdateLogEntry::StartTime(t) => e.exec_start_at = Some(t),
                    });
                }
                Message::CreateConfigQueryLog {
                    query_id,
                    query_text,
                    generated_ncl,
                    diagnostics_file,
                } => {
                    let content = fs::read_to_string(&diagnostics_file)
                        .expect("Failed to read diagnostics file");
                    self.create_udi_pgp_set_record(
                        query_id,
                        query_text,
                        generated_ncl,
                        diagnostics_file,
                        content,
                    )
                }
            }
        }
    }
}
