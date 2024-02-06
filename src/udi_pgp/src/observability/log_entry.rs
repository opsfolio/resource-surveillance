use std::io::{self, Write};

use serde::{Deserialize, Serialize};
use tracing::field::{Field, Visit};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Elaboration {
    pub events: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct QueryLogEntry {
    pub query_id: String,
    pub query_text: String,
    pub exec_start_at: Option<String>,
    pub exec_finish_at: Option<String>,
    pub elaboration: Elaboration,
    pub exec_msg: Vec<String>,
}

impl QueryLogEntry {
    pub fn new(query: &str) -> Self {
        let id = Uuid::new_v4();
        QueryLogEntry {
            query_id: id.to_string(),
            query_text: query.to_string(),
            exec_start_at: None,
            exec_finish_at: None,
            exec_msg: vec![],
            elaboration: Elaboration::default(),
        }
    }
}

impl Write for QueryLogEntry {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let input =
            std::str::from_utf8(buf).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // The format is
        // - "query_id:<id>"
        // - "query_tex:text"
        if let Some((field, text)) = input.split_once(':') {
            match field {
                "query_id" => self.query_id = text.to_string(),
                "query_text" => self.query_text = text.to_string(),
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Expected field names, `query_id` and `query_text` only",
                    ))
                }
            }
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Input must contain ':'",
            ));
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

impl Visit for QueryLogEntry {
    fn record_debug(&mut self, field: &Field, value: &dyn std::fmt::Debug) {
        match field.name() {
            "query_id" => self.query_id = format!("{:?}", value),
            "query_text" => self.query_text = format!("{:?}", value),
            _ => {}
        };
    }
}
