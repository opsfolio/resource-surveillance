//! # UDI-PGP State Handler
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

use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};
use tracing::{debug, error, Level};

use crate::{config::UdiPgpConfig, observability::QueryLogEntryMap};

use self::messages::{Message, UpdateLogEntry};

mod database;
pub mod messages;

/// State Manager
pub struct StateManager {
    config: Arc<Mutex<UdiPgpConfig>>,
    log_entries: Arc<Mutex<QueryLogEntryMap>>,
}

impl StateManager {
  /// Initialize the state manager
    pub fn init(config: Arc<Mutex<UdiPgpConfig>>, entries: Arc<Mutex<QueryLogEntryMap>>) -> Self {
        StateManager {
            config,
            log_entries: entries,
        }
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
                    debug!("Updated Core Config Successfully")
                }
                Message::InsertSupplier(id, supplier) => {
                    debug!(
                        "Updating suppliers with supplier_id: {id} and supplier: {:#?}",
                        supplier
                    );
                    let mut config = shared_config.lock().await;
                    config.suppliers.insert(id, supplier);
                    debug!("Supplier updated successfully",);
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
                        }
                        UpdateLogEntry::StartTime(t) => e.exec_start_at = Some(t),
                    });
                }
            }
        }
    }
}
