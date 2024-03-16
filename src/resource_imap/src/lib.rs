use std::{collections::HashMap, net::TcpStream, sync::Arc, vec};

use anyhow::{anyhow, Context};
use imap::types::Fetch;
use rustls::RootCertStore;
use serde::{Deserialize, Serialize};

use mail_parser::{Message, MessageParser, MimeHeaders, PartType};

mod msft;

pub use msft::{Microsoft365AuthServerConfig, Microsoft365Config, TokenGenerationMethod};
use tracing::{debug, error};

use crate::msft::retrieve_emails;

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

#[derive(Serialize, Deserialize, Debug)]
pub struct ImapConfig {
    pub username: Option<String>,
    pub password: Option<String>,
    pub addr: Option<String>,
    pub port: u16,
    pub folder: String,
    pub mailboxes: Vec<String>,
    pub batch_size: u64,
    pub extract_attachments: bool,
    /// The select query name is first, followed by the selector itself
    pub css_selectors: Vec<(String, String)>,
    pub microsoft365: Option<Microsoft365Config>,
}

/// Traverses through each mailbox/folder, processes the email and extracts details.
/// Returns a hashmap containing each folder processed whuich points to all the emails in that folder.
extern crate imap;

pub async fn process_imap(
    config: &mut ImapConfig,
) -> anyhow::Result<HashMap<String, Vec<EmailResource>>> {
    debug!("{config:#?}");

    if let Some(msft_config) = config.microsoft365.clone() {
        return retrieve_emails(&msft_config, config)
            .await
            .with_context(|| "[ingest_imap]: microsoft_365. Failed to retrieve emails");
    }

    let mut root_store = RootCertStore::empty();
    root_store.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());

    let mut client_config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();
    client_config.key_log = Arc::new(rustls::KeyLogFile::new());

    let server_name = config.addr.clone().unwrap().clone().try_into()?;
    let mut conn = rustls::ClientConnection::new(Arc::new(client_config), server_name)?;
    let mut sock = TcpStream::connect(format!("{}:{}", config.addr.clone().unwrap(), config.port))?;
    let tls = rustls::Stream::new(&mut conn, &mut sock);

    let client = imap::Client::new(tls);

    let mut imap_session = client
        .login(
            &config.username.clone().unwrap(),
            &config.password.clone().unwrap(),
        )
        .map_err(|e| e.0)?;

    let mailboxes = imap_session.list(None, Some(&config.folder))?;
    config.mailboxes = mailboxes
        .into_iter()
        .map(|m| m.name().to_string())
        .collect();

    let mut res = HashMap::new();
    for folder in &config.mailboxes {
        debug!("Processing messages in {} folder", folder);
        match imap_session.select(folder) {
            Ok(mailbox) => {
                let messages_total = mailbox.exists;
                debug!("Number of messages in folder: {messages_total}");
                if messages_total == 0 {
                    error!("No messages in {} folder", folder);
                    continue;
                }

                // get no of batches and the size of each batch
                let mut remaining_emails =
                    std::cmp::min(config.batch_size as usize, messages_total as usize);
                let mut start = messages_total as usize;
                // Max number of emails to fetch per batch because of IMAP limitations
                let batch_size = 1000; 
                let mut emails = Vec::new();

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

                    let fetched_messages = imap_session.fetch(fetch_range, "RFC822")?;
                    for message in fetched_messages.iter() {
                        let email = convert_to_email_resource(message, config)?;
                        emails.push(email);
                    }

                    remaining_emails = remaining_emails.saturating_sub(fetch_size);
                    if start == 0 {
                        break;
                    }
                }

                res.insert(folder.clone(), emails);
            }
            Err(err) => {
                error!(
                    "Error selecting folder '{}': {}. Skipping this folder.",
                    folder, err
                );
                continue;
            }
        }
    }

    Ok(res)
}

fn convert_to_email_resource(
    message: &Fetch,
    config: &ImapConfig,
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
        cc: parse_addresses(message.cc()),
        bcc: parse_addresses(message.bcc()),
        references: vec![],
        in_reply_to: None,
        message_id: message.message_id().unwrap_or_default().to_string(),
        to: parse_addresses(message.to()),
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
        attachments: if config.extract_attachments {
            Some(extract_attachments(&message))
        } else {
            None
        },
    };

    Ok(email)
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

fn extract_attachments(message: &Message) -> Vec<Attachment> {
    let mut attachments = Vec::new();
    extract_attachments_recursive(message, &mut attachments);
    attachments
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
            extract_attachments_recursive(inner_message, attachments);
        }
    }
}
