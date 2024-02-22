use std::{collections::HashMap, vec};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use mail_parser::{Message, MessageParser, MimeHeaders};

#[derive(Serialize, Deserialize, Debug)]
pub struct Attachment {
    filename: String,
    content_type: String,
    content: Vec<u8>,
    content_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailResource {
    subject: String,
    from: String,
    cc: Vec<String>,
    bcc: Vec<String>,
    references: Vec<String>,
    in_reply_to: Option<String>,
    pub message_id: String,
    to: Vec<String>,
    pub date: String,
    text_plain: Vec<String>,
    text_html: Vec<String>,
    pub raw_text: String,
    pub raw_json: String,
    attachments: Option<Vec<Attachment>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImapConfig {
    pub username: String,
    pub password: String,
    pub addr: String,
    pub port: u16,
    pub folders: Vec<String>,
    pub max_no_messages: u64,
    pub extract_attachments: bool,
}

/// Traverses through each mailbox/folder, processes the email and extracts details.
/// Returns a hashmap containing each folder processed whuich points to all the emails in that folder.

pub fn process_imap(config: &ImapConfig) -> anyhow::Result<HashMap<String, Vec<EmailResource>>> {
    let tls = native_tls::TlsConnector::builder().build()?;
    let client = imap::connect((config.addr.clone(), config.port), &config.addr, &tls)?;
    let mut imap_session = client
        .login(&config.username, &config.password)
        .map_err(|e| e.0)?;

    let mut res = HashMap::new();

    for folder in &config.folders {
        let mailbox = imap_session.select(folder)?;
        let messages_total = mailbox.exists;
        let start =
            messages_total.saturating_sub(config.max_no_messages.saturating_sub(1).try_into()?);
        let fetch_range = format!("{}:*", std::cmp::max(start, 1));

        let messages = imap_session.fetch(fetch_range, "RFC822")?;
        let mut emails = Vec::new();

        for message in messages.iter() {
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
                text_plain: message.text_bodies().map(|s| s.to_string()).collect(),
                text_html: message.html_bodies().map(|s| s.to_string()).collect(),
                raw_text: String::from_utf8_lossy(message.raw_message()).into_owned(),
                raw_json: serde_json::to_string(&message)?,
                attachments: if config.extract_attachments {
                    Some(extract_attachments(&message))
                } else {
                    None
                },
            };

            emails.push(email);
        }
        res.insert(folder.clone(), emails);
    }

    Ok(res)
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
