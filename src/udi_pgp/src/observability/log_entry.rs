use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryLogEntry {
    pub query_id: String,
    pub query_text: String,
    pub exec_start_at: Option<String>,
    pub exec_finish_at: Option<String>,
    pub events: Vec<String>,
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
            events: vec![],
            exec_msg: vec![],
        }
    }
}
