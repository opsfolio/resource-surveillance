use async_trait::async_trait;
use serde::{Deserialize, Serialize};

mod default_imap_service;
pub mod elaboration;
mod msft;

pub use msft::{Microsoft365AuthServerConfig, TokenGenerationMethod, Microsoft365Config};
use tracing::debug;

use crate::{default_imap_service::DefaultImapService, msft::MicrosoftImapResource};

#[async_trait]
pub trait ImapResource {
    // Checks if progress is enabled
    fn progress(&mut self) -> bool;
    /// Initiate the IMAP Client, perform necessary login
    async fn init(&mut self) -> anyhow::Result<()>;
    /// List all available folders in the mailbox
    async fn folders(&mut self) -> anyhow::Result<Vec<String>>;
    /// Get all folders as described by the user, i.e. the `--f` argument
    async fn specified_folders(&mut self, folder_pattern: &str) -> anyhow::Result<Vec<Folder>>;
    /// Get messages in that folder
    async fn process_messages_in_folder(&mut self, folder: &mut Folder) -> anyhow::Result<()>;
    /// Username of the IMAP server
    fn username(&mut self) -> String;
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    filename: String,
    content_type: String,
    content: Vec<u8>,
    content_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailResource {
    pub subject: String,
    pub from: String,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub references: Vec<String>,
    in_reply_to: Option<String>,
    pub message_id: String,
    pub to: Vec<String>,
    pub date: String,
    pub text_plain: Vec<String>,
    pub text_html: Vec<String>,
    pub raw_text: String,
    pub raw_json: String,
    attachments: Option<Vec<Attachment>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImapConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub addr: Option<String>,
    pub port: u16,
    pub folder: String,
    pub mailboxes: Vec<String>,
    pub batch_size: u64,
    pub extract_attachments: bool,
    pub microsoft365: Option<Microsoft365Config>,
    pub progress: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Folder {
    pub name: String,
    pub metadata: serde_json::Value,
    pub messages: Vec<EmailResource>,
}

impl From<String> for Folder {
    fn from(value: String) -> Self {
        Folder {
            name: value,
            metadata: serde_json::Value::Null,
            messages: vec![],
        }
    }
}

impl Folder {
    pub fn metadata(&mut self, value: serde_json::Value) {
        self.metadata = value
    }

    pub fn messages(&mut self, msgs: Vec<EmailResource>) {
        self.messages = msgs
    }
}

pub async fn imap(config: &ImapConfig) -> anyhow::Result<Box<dyn ImapResource>> {
    debug!("{config:#?}");

    Ok(match &config.microsoft365 {
        Some(microsoft) => {
            let mut msft_resource = MicrosoftImapResource::new(
                &microsoft.client_id,
                &microsoft.client_secret,
                microsoft.mode.clone(),
                config,
            );
            msft_resource.server(microsoft.auth_server.clone());
            msft_resource.redirect_uri(microsoft.redirect_uri.clone());

            Box::new(msft_resource)
        }
        None => Box::new(DefaultImapService::new(config.clone())),
    })
}
