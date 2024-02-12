use std::net::SocketAddr;
use tokio::sync::oneshot;
use tracing::{span, Level};

use crate::{
    config::{Supplier, UdiPgpConfig},
    observability::{log_entry::QueryLogEntry, QueryLogEntryMap},
};

pub enum UpdateLogEntry {
    StartTime(String),
    EndTime(String),
    Event(String, Level),
}

pub enum Message {
    /// Updates the metrics and health addresses
    UpdateCore(Option<SocketAddr>, Option<SocketAddr>),
    /// Adds a new supplier to the configuration
    InsertSupplier(String, Supplier),
    ReadConfig(oneshot::Sender<UdiPgpConfig>),
    ReadLogEntries(oneshot::Sender<QueryLogEntryMap>),
    AddLogEntry {
        log: QueryLogEntry,
        span_id: span::Id,
    },
    UpdateLogEntry {
        span_id: span::Id,
        msg: UpdateLogEntry,
    },
}
