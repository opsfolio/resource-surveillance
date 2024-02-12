use std::net::SocketAddr;
use tokio::sync::oneshot;
use tracing::{span, Level};

use crate::{
    config::{Supplier, UdiPgpConfig},
    observability::{log_entry::QueryLogEntry, QueryLogEntryMap},
};

/// Update the start, end times and the events of an entry
pub enum UpdateLogEntry {
    /// Start of execution for the query
    StartTime(String),
    /// Query ends
    EndTime(String),
    /// Add an event with a level. The level distinguishes if it should be recored in `exec_msg`
    /// and denotes the `exec_status` if it is an error.
    Event(String, Level),
}

pub enum Message {
    /// Updates the metrics and health addresses
    UpdateCore(Option<SocketAddr>, Option<SocketAddr>),
    /// Adds a new supplier to the configuration
    InsertSupplier(String, Supplier),
    /// Get the configuration
    ReadConfig(oneshot::Sender<UdiPgpConfig>),
    /// List all entires
    ReadLogEntries(oneshot::Sender<QueryLogEntryMap>),
    /// Insert a log entry into memorry.
    /// The log entry only gets inserted in the database after it ends.
    AddLogEntry {
        /// The log to instert
        log: QueryLogEntry,
        /// The id of the span
        span_id: span::Id,
    },
    /// Update the start, end times and the events of an entry
    UpdateLogEntry {
        /// The id of the span
        span_id: span::Id,
        msg: UpdateLogEntry,
    },
    /// Create a record for SET query, i.e a config query
    CreateConfigQueryLog {
        query_id: String,
        query_text: String,
        generated_ncl: String,
        diagnostics_file: String,
    }
}
