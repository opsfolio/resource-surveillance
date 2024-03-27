use anyhow::{anyhow, Context};
use async_imap::{
    types::{Fetch, Mailbox},
    Session,
};
use async_trait::async_trait;
use futures_util::TryStreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use mail_parser::{Message, MessageParser, MimeHeaders, PartType};
use std::{fmt::Debug, sync::Arc};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::{
    client::TlsStream,
    rustls::{ClientConfig, RootCertStore},
};

use tracing::debug;

use crate::{Attachment, EmailResource, Folder, ImapConfig, ImapResource};

#[async_trait]
trait SessionAbstraction: Debug + Send + Sync {
    async fn list_folders(
        &mut self,
        ref_name: Option<&str>,
        folder_pattern: Option<&str>,
    ) -> anyhow::Result<Vec<String>>;
    async fn select_folder(&mut self, folder_name: &str) -> anyhow::Result<Mailbox>;
    async fn fetch_messages_from_folder(
        &mut self,
        sequence_set: &str,
    ) -> anyhow::Result<Vec<Fetch>>;
    async fn specified_folders(
        &mut self,
        ref_name: Option<&str>,
        folder_pattern: Option<&str>,
    ) -> anyhow::Result<Vec<Folder>>;
}

#[derive(Debug)]
struct SessionHolder {
    session: Session<TlsStream<TcpStream>>,
}

#[async_trait]
impl SessionAbstraction for SessionHolder {
    async fn list_folders(
        &mut self,
        ref_name: Option<&str>,
        folder_pattern: Option<&str>,
    ) -> anyhow::Result<Vec<String>> {
        let mailboxes = self.session.list(ref_name, folder_pattern).await?;
        let mailboxes: Vec<_> = mailboxes.try_collect().await?;
        Ok(mailboxes.iter().map(|m| m.name().to_string()).collect())
    }

    async fn specified_folders(
        &mut self,
        ref_name: Option<&str>,
        folder_pattern: Option<&str>,
    ) -> anyhow::Result<Vec<Folder>> {
        let mailboxes = self.session.list(ref_name, folder_pattern).await?;
        let mailboxes: Vec<_> = mailboxes.try_collect().await?;
        Ok(mailboxes
            .into_iter()
            .map(|m| Folder::from(m.name().to_string()))
            .collect())
    }

    async fn select_folder(&mut self, folder_name: &str) -> anyhow::Result<Mailbox> {
        Ok(self.session.select(folder_name).await?)
    }

    // TODO: add error message on how to select the errors
    async fn fetch_messages_from_folder(
        &mut self,
        sequence_set: &str,
    ) -> anyhow::Result<Vec<Fetch>> {
        let messsages_stream = self.session.fetch(sequence_set, "RFC822").await?;
        Ok(messsages_stream.try_collect().await?)
    }
}

/// This is the default IMAP service that utilizes the `rust-imap` library
#[derive(Debug)]
pub struct DefaultImapService {
    username: String,
    password: String,
    addr: String,
    port: u16,
    batch_size: u64,
    extract_attachments: bool,
    session: Option<Box<dyn SessionAbstraction>>,
    // session: Option<Session<TlsStream<TcpStream>>>,
    progress: Option<ProgressBar>,
}

impl DefaultImapService {
    pub fn new(value: ImapConfig) -> Self {
        DefaultImapService {
            username: value.username.expect("Expected username"),
            password: value.password.expect("Expected Password"),
            addr: value.addr.expect("Failed to get address"),
            port: value.port,
            batch_size: value.batch_size,
            extract_attachments: value.extract_attachments,
            session: None,
            progress: if value.progress {
                Some(ProgressBar::new_spinner())
            } else {
                None
            },
        }
    }

    fn session_mut(&mut self) -> &mut Box<dyn SessionAbstraction> {
        self.session.as_mut().expect("Session is not initialized")
    }

    fn update_progress(&mut self, message: String, style: bool) -> anyhow::Result<()> {
        if let Some(spinner) = &self.progress {
            spinner.set_message(message);
            if style {
                spinner.set_style(
                    ProgressStyle::default_spinner()
                        .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                        .template("{spinner:.blue} {msg}")?,
                );
            }
        }
        Ok(())
    }

    fn parse_addresses(addr: Option<&mail_parser::Address>) -> Vec<String> {
        match addr {
            None => vec![],
            Some(addrs) => addrs
                .clone()
                .into_list()
                .iter()
                .map(|a| {
                    a.address()
                        .as_ref()
                        .map_or("".to_string(), ToString::to_string)
                })
                .collect(),
        }
    }

    fn extract_attachments_recursive(message: &Message, attachments: &mut Vec<Attachment>) {
        for attachment in message.attachments() {
            if !attachment.is_message() {
                let name = attachment
                    .attachment_name()
                    .unwrap_or("Untitled")
                    .to_string();
                let file_type = attachment
                    .content_type()
                    .map(|ct| ct.c_type.to_string())
                    .unwrap_or_else(|| ".txt".to_string());
                let content = attachment.contents().to_vec();
                let id = attachment.content_id().unwrap_or_default().to_string();
                attachments.push(Attachment {
                    filename: name,
                    content_type: file_type,
                    content,
                    content_id: id,
                });
            } else if let Some(inner_message) = attachment.message() {
                Self::extract_attachments_recursive(inner_message, attachments);
            }
        }
    }

    fn extract_attachments(message: &Message) -> Vec<Attachment> {
        let mut attachments = Vec::new();
        Self::extract_attachments_recursive(message, &mut attachments);
        attachments
    }

    fn convert_to_email_resource(
        message: &Fetch,
        extract_attachments: bool,
    ) -> anyhow::Result<EmailResource> {
        let body = message
            .body()
            .ok_or_else(|| anyhow!("Message did not have a body"))?;

        let message = MessageParser::default()
            .parse(body)
            .ok_or_else(|| anyhow!("Failed to parse email message"))?;

        let email = EmailResource {
            subject: message.subject().unwrap_or_default().to_string(),
            from: message
                .from()
                .map(|addresses| {
                    addresses
                        .first()
                        .unwrap()
                        .address
                        .clone()
                        .unwrap_or_default()
                })
                .unwrap_or_default()
                .to_string(),
            cc: Self::parse_addresses(message.cc()),
            bcc: Self::parse_addresses(message.bcc()),
            references: vec![],
            in_reply_to: None,
            message_id: message.message_id().unwrap_or_default().to_string(),
            to: Self::parse_addresses(message.to()),
            date: message.date().map(|d| d.to_rfc3339()).unwrap_or_default(),
            text_plain: message
                .text_bodies()
                .map(|s| match &s.body {
                    PartType::Text(txt) => txt.to_string(),
                    _ => "".to_string(),
                })
                .collect(),
            text_html: message
                .html_bodies()
                .map(|s| match &s.body {
                    PartType::Html(html) => html.to_string(),
                    _ => "".to_string(),
                })
                .collect(),
            raw_text: String::from_utf8_lossy(message.raw_message()).into_owned(),
            raw_json: serde_json::to_string(&message)?,
            attachments: if extract_attachments {
                Some(Self::extract_attachments(&message))
            } else {
                None
            },
        };

        Ok(email)
    }
}

#[async_trait]
impl ImapResource for DefaultImapService {
    async fn init(&mut self) -> anyhow::Result<()> {
        let mut root_store = RootCertStore::empty();
        root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

        let client_config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_no_client_auth();
        let config_ref = Arc::new(client_config);
        let server_name = self.addr.clone().try_into()?;

        let conn = TlsConnector::from(config_ref.clone());
        let stream = TcpStream::connect(format!("{}:{}", self.addr, self.port)).await?;
        let mut tls = conn.connect(server_name, stream).await?;

        let client = async_imap::Client::new(tls);

        let session = client
            .login(&self.username, &self.password)
            .await
            .map_err(|e| e.0)?;

        self.session = Some(Box::new(SessionHolder { session }));

        Ok(())
    }

    fn progress(&mut self) -> bool {
        self.progress.is_some()
    }

    fn username(&mut self) -> String {
        self.username.to_string()
    }

    async fn folders(&mut self) -> anyhow::Result<Vec<String>> {
        let sess = self.session_mut();
        sess.list_folders(None, Some("*")).await
    }

    async fn specified_folders(
        &mut self,
        folder_pattern: &str,
    ) -> anyhow::Result<Vec<crate::Folder>> {
        let sess = self.session_mut();
        sess.specified_folders(None, Some(folder_pattern)).await
    }

    async fn process_messages_in_folder(
        &mut self,
        folder: &mut crate::Folder,
    ) -> anyhow::Result<()> {
        let batch_size = self.batch_size;
        let extract_attachments = self.extract_attachments;

        self.update_progress(
            format!("Downloading messages from folder: {}", folder.name),
            true,
        )?;

        let mailbox = {
            let sess = self.session_mut();
            sess.select_folder(&folder.name)
                .await
                .with_context(|| format!("Failed to select {} folder", folder.name))?
        };
        let folder_metadata = serde_json::to_value(mailbox.to_string())?;
        folder.metadata(folder_metadata);

        let messages_total = mailbox.exists;

        debug!("Number of messages in folder: {messages_total}");
        if messages_total == 0 {
            eprintln!("No messages in {} folder", folder.name);
            return Ok(());
        }

        // get no of batches and the size of each batch
        let mut remaining_emails = std::cmp::min(batch_size as usize, messages_total as usize);
        let mut start = messages_total as usize;
        // Max number of emails to fetch per batch because of IMAP limitations
        let batch_size = 1000;
        let mut emails = Vec::new();

        // TODO: do this part outside, fetch only the batch, then refetch if less
        while remaining_emails > 0 {
            let fetch_size = std::cmp::min(remaining_emails, batch_size);
            let end = start;
            start = start.saturating_sub(fetch_size);

            let fetch_range = if start > 0 {
                format!("{}:{}", start, end)
            } else {
                format!("1:{}", end)
            };
            debug!("Fetching emails in range: {fetch_range}");

            self.update_progress(
                format!("Fetching {} messages from {}", fetch_size, folder.name),
                false,
            )?;

            {
                let sess = self.session_mut();
                let fetched_messages = sess.fetch_messages_from_folder(&fetch_range).await?;
                for message in fetched_messages.iter() {
                    let email = Self::convert_to_email_resource(message, extract_attachments)?;
                    emails.push(email);
                }
            }

            remaining_emails = remaining_emails.saturating_sub(fetch_size);
            if start == 0 {
                break;
            }

            if self.progress() {
                let spinner = &self.progress.as_ref().unwrap();
                spinner.inc(1);
            }
        }

        if self.progress() {
            let spinner = &self.progress.as_ref().unwrap();
            spinner.finish_with_message(format!(
                "Fetched {} from {} folder successfully",
                emails.len(),
                folder.name
            ));
        }
        folder.messages(emails);

        Ok(())
    }
}
