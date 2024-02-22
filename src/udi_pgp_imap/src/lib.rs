use std::collections::HashMap;

use mailparse::{parse_mail, MailHeaderMap};
use serde::{Deserialize, Serialize};

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
            let body_str = std::str::from_utf8(body)
                .expect("message was not valid utf-8")
                .to_string();

            let parsed_mail = parse_mail(body_str.as_bytes())?;

            let email = EmailResource {
                raw_text: body_str.to_string(),
                subject: parsed_mail
                    .headers
                    .get_first_value("Subject")
                    .unwrap_or_default(),
                from: parsed_mail
                    .headers
                    .get_first_value("From")
                    .unwrap_or_default(),
                to: parsed_mail
                    .headers
                    .get_first_value("To")
                    .unwrap_or_default(),
                cc: parsed_mail
                    .headers
                    .get_first_value("Cc")
                    .unwrap_or_default(),
                bcc: parsed_mail
                    .headers
                    .get_first_value("Bcc")
                    .unwrap_or_default(),
                references: parsed_mail
                    .headers
                    .get_first_value("References")
                    .unwrap_or_default(),
                in_reply_to: parsed_mail
                    .headers
                    .get_first_value("In-Reply-To")
                    .unwrap_or_default(),
                message_id: parsed_mail
                    .headers
                    .get_first_value("Message-ID")
                    .unwrap_or_default(),
                date: parsed_mail
                    .headers
                    .get_first_value("Date")
                    .unwrap_or_default(),
                text_plain: parsed_mail
                    .subparts
                    .iter()
                    .filter_map(|p| {
                        if p.ctype.mimetype == "text/plain" {
                            p.get_body().ok()
                        } else {
                            None
                        }
                    })
                    .collect(),
                text_html: parsed_mail
                    .subparts
                    .iter()
                    .filter_map(|p| {
                        if p.ctype.mimetype == "text/html" {
                            p.get_body().ok()
                        } else {
                            None
                        }
                    })
                    .collect(),
            };
            emails.push(email);
        }

        res.insert(folder.to_string(), emails);
    }

    Ok(res)
}
