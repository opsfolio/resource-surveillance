use std::{collections::HashMap, fmt::Debug, fmt::Write};

use chrono::prelude::*;
use derive_new::new;
use tokio::sync::{mpsc, oneshot};
use tracing::field::{Field, Visit};
use tracing::subscriber::set_global_default;
use tracing::{debug, error, span, Event, Id, Level, Subscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::registry::LookupSpan;
use tracing_subscriber::{fmt, EnvFilter, Layer, Registry};

use crate::config::manager::{Message, UpdateLogEntry};
use crate::error::{UdiPgpError, UdiPgpResult};

use self::log_entry::QueryLogEntry;
pub mod log_entry;
pub type QueryLogEntryMap = HashMap<Id, QueryLogEntry>;

pub struct StringVisitor<'a> {
    string: &'a mut String,
}

impl<'a> Visit for StringVisitor<'a> {
    fn record_debug(&mut self, _field: &Field, value: &dyn std::fmt::Debug) {
        write!(self.string, "{:?} ", value).unwrap();
    }
}

#[derive(Debug, new)]
struct UdiPgpTracingLayer {
    state_tx: mpsc::Sender<Message>,
}

impl UdiPgpTracingLayer {
    async fn _read_log_entries(&self) -> UdiPgpResult<QueryLogEntryMap> {
        let (response_tx, response_rx) = oneshot::channel();
        let read_state_msg = Message::ReadLogEntries(response_tx);
        self.state_tx
            .send(read_state_msg)
            .await
            .expect("Failed to send message");
        match response_rx.await {
            Ok(logs) => Ok(logs),
            Err(e) => {
                error!("{}", e);
                Err(UdiPgpError::ConfigError(format!(
                    "Failed to read log entries: {}",
                    e
                )))
            }
        }
    }
}

impl<S> Layer<S> for UdiPgpTracingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_event(&self, event: &Event<'_>, ctx: tracing_subscriber::layer::Context<'_, S>) {
        let span = ctx.current_span();
        if let Some(span_ref) = span.id().and_then(|id| ctx.span(id)) {
            debug!("An event in span {:#?} happened", span_ref.id());

            let state_tx = self.state_tx.clone();
            let id = span_ref.id().clone();

            let mut event_msg = String::new();
            let mut visitor = StringVisitor {
                string: &mut event_msg,
            };
            event.record(&mut visitor);
            let event_msg = event_msg.trim().to_string();

            tokio::spawn(async move {
                let msg = Message::UpdateLogEntry {
                    span_id: id,
                    msg: UpdateLogEntry::Event(event_msg),
                };
                if let Err(e) = state_tx.send(msg).await {
                    error!("Failed to add event to log entry: {}", e);
                }
            });
        }
    }

    fn on_new_span(
        &self,
        attrs: &span::Attributes<'_>,
        id: &span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        debug!("New span {:#?} created", id);

        let mut query = String::new();
        let mut visitor = StringVisitor { string: &mut query };
        attrs.record(&mut visitor);
        let query = query.trim();
        debug!("New span attrs {:#?} created", query);

        let state_tx = self.state_tx.clone();
        let id_clone = id.clone();
        let entry = QueryLogEntry::new(query);

        // TODO: look into this as it adds quite the overhead.
        tokio::spawn(async move {
            let msg = Message::AddLogEntry {
                log: entry,
                span_id: id_clone,
            };
            if let Err(e) = state_tx.send(msg).await {
                error!("Failed to add log entry: {}", e);
            }
        });
    }

    fn on_enter(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        debug!("Span {:#?} was entered at time: {now}", id);

        let state_tx = self.state_tx.clone();
        let id = id.clone();
        tokio::spawn(async move {
            let msg = Message::UpdateLogEntry {
                span_id: id,
                msg: UpdateLogEntry::StartTime(now),
            };
            if let Err(e) = state_tx.send(msg).await {
                error!("Failed to update start time for log: {}", e);
            }
        });
    }

    fn on_exit(&self, id: &span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S").to_string();
        debug!("Span {:#?} exited at time: {now}", id);

        let state_tx = self.state_tx.clone();
        let id = id.clone();
        tokio::spawn(async move {
            let msg = Message::UpdateLogEntry {
                span_id: id,
                msg: UpdateLogEntry::EndTime(now),
            };
            if let Err(e) = state_tx.send(msg).await {
                error!("Failed to updated end log time: {}", e);
            }
        });
    }

    fn on_close(&self, id: span::Id, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        debug!("Span {:#?} closed", id)
    }
}

pub fn init(state_tx: &mpsc::Sender<Message>, verbose: bool) -> anyhow::Result<()> {
    let level = if verbose { Level::DEBUG } else { Level::INFO };
    let env_filter = EnvFilter::new(level.to_string());

    let fmt_layer = fmt::layer().compact().with_line_number(true);

    let subscriber = Registry::default()
        .with(env_filter)
        .with(UdiPgpTracingLayer::new(state_tx.clone())) // Assuming UdiPgpTracingLayer is your custom layer
        .with(fmt_layer);

    set_global_default(subscriber)?;
    Ok(())
}
