use std::{collections::HashMap, vec};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use mail_parser::MessageParser;

#[derive(Serialize, Deserialize, Debug)]
pub struct EmailResource {
    subject: String,
    from: String,
    cc: String,
    bcc: String,
    references: String,
    in_reply_to: String,
    pub message_id: String,
    to: String,
    pub date: String,
    text_plain: Vec<String>,
    text_html: Vec<String>,
    pub raw_text: String,
    pub raw_json: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ImapConfig {
    pub username: String,
    pub password: String,
    pub addr: String,
    pub port: u16,
    pub folders: Vec<String>,
    pub max_no_messages: u64,
}

/// Traverses through each mailbox/folder, processes the email and extracts details.
/// Returns a hashmap containing each folder processed whuich points to all the emails in that folder.
pub fn process_imap(config: &ImapConfig) -> anyhow::Result<HashMap<String, Vec<EmailResource>>> {
    let tls = native_tls::TlsConnector::builder().build()?;

    // Connect to the Gmail IMAP server
    let client = imap::connect((config.addr.clone(), config.port), &config.addr, &tls)?;
    let mut imap_session = client
        .login(&config.username, &config.password)
        .map_err(|e| e.0)?;

    let mut res = HashMap::new();

    for folder in &config.folders {
        let mailbox = imap_session.select(folder)?;

        // TODO: remove this or replace with max_no
        let messages_total = mailbox.exists;
        // println!("{:#?}", messages_total);

        // select the last 5 emails
        let start = if messages_total >= 5 {
            messages_total - 4
        } else {
            1
        };
        let fetch_range = format!("{}:*", start);

        let messages = imap_session.fetch(fetch_range, "RFC822")?;
        let mut emails = Vec::new();

        for message in messages.iter() {
            let body = message.body().expect("message did not have a body!");

            let message = MessageParser::default()
                .parse(body)
                .ok_or_else(|| anyhow!("Failed to parse email message"))?;

            let raw_text = std::str::from_utf8(message.raw_message())
                .expect("message was not valid utf-8")
                .to_string();
            let raw_json = serde_json::to_string(&message).unwrap();

            let email = EmailResource {
                subject: message.subject().unwrap_or_default().to_string(),
                from: message
                    .from()
                    .unwrap()
                    .first()
                    .unwrap()
                    .address
                    .clone()
                    .unwrap()
                    .to_string(),
                cc: "".to_string(),
                bcc: "".to_string(),
                references: "".to_string(),
                in_reply_to: "".to_string(),
                message_id: message.message_id().unwrap().to_string(),
                to: "".to_string(),
                date: "".to_string(),
                text_plain: vec!["".to_string()],
                text_html: vec!["".to_string()],
                raw_text,
                raw_json,
            };
            emails.push(email);
        }
        res.insert(folder.to_string(), emails);
    }

    Ok(res)
}
