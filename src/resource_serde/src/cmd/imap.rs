use clap::{Args, ValueEnum};
use serde::Serialize;
const DEFAULT_STATEDB_FS_PATH: &str = "resource-surveillance.sqlite.db";

#[derive(Debug, Serialize, Clone, ValueEnum, Default)]
pub enum ImapMessageStatus {
    #[default]
    Unread,
    Read,
    Starred,
}

/// Ingest content from email boxes
#[derive(Debug, Serialize, Args, Clone)]
pub struct IngestImapArgs {
    /// target SQLite database
    #[arg(short='d', long, default_value = DEFAULT_STATEDB_FS_PATH, default_missing_value = "always", env="SURVEILR_STATEDB_FS_PATH")]
    pub state_db_fs_path: String,

    /// email address
    #[arg(short, long)]
    pub username: String,

    /// password to the email. mainly an app password.
    /// See the documentation on how to create an app password
    #[arg(short, long)]
    pub password: String,

    /// IMAP server address. e.g imap.google.com or outlook.office365.com
    #[arg(short = 'a', long)]
    pub server_addr: String,

    /// IMAP server port.
    #[arg(long, default_value = "993")]
    pub port: u16,

    /// Mailboxes to read from. i.e folders.
    #[arg(short, long, default_value = "INBOX")]
    pub folders: Vec<String>,

    /// Status of the messages to be ingested.
    #[arg(short, long, default_value = "UNREAD")]
    pub status: Vec<ImapMessageStatus>,

    /// Maximum number of messages to be ingested.
    #[arg(short, long, default_value = "100")]
    pub max_no_messages: u64,
}
