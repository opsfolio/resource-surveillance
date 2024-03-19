use std::collections::HashMap;

use resource_imap::ImapConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
/// Elaboration on the events and happenings while fetching emails from the server and ingesting
pub struct ImapElaboration {
    /// The configuration, which include the CLI arguments
    pub imap_configuration: ImapConfig,
    /// Time it took to fetch/download raw emails the email server
    pub email_fetch_duration: Option<String>,
    /// Total time it took to ingest the email, same as the difference between start and end time in `ur_ingest_session`
    pub email_ingest_duration: Option<String>,
    /// Number of folders discovered
    pub discovered_folder_count: usize,
    /// Folder elaborations
    pub folders: HashMap<String, FolderElaboration>,
}

impl ImapElaboration {
    pub fn new(config: &ImapConfig) -> Self {
        Self {
            imap_configuration: config.clone(),
            email_fetch_duration: None,
            email_ingest_duration: None,
            discovered_folder_count: 0,
            folders: HashMap::new(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
/// Folder details and elaboration
pub struct FolderElaboration {
    /// Name of the folder
    pub name: String,
    /// Total number of messages fetched in the folder
    pub fetched_message_count: usize,
    /// Time it took for the folder to get processed
    pub folder_process_duration: Option<String>,
    /// Total number of text/plain contents across all emails in the folder
    pub text_plain_count: usize,
    /// Total number of text/html content encountered for all emails in the folder
    pub html_content_count: usize,
}

impl FolderElaboration {
    pub fn new(name: &str, messages: usize) -> Self {
        FolderElaboration {
            name: name.to_string(),
            fetched_message_count: messages,
            folder_process_duration: None,
            text_plain_count: 0,
            html_content_count: 0,
        }
    }
}
