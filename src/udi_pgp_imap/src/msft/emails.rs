use std::collections::HashMap;

use crate::{EmailResource, ImapConfig};
use anyhow::Context;
use graph_rs_sdk::{oauth::AccessToken, Graph, ODataQuery};
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MailFolder {
    id: String,
    display_name: String,
    parent_folder_id: String,
    child_folder_count: usize,
    unread_item_count: usize,
    total_item_count: usize,
    size_in_bytes: usize,
    is_hidden: bool,
}

#[derive(Serialize, Deserialize, Debug)]
struct MessageList {
    value: Vec<Message>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Message {
    id: String,
    created_date_time: String,
    last_modified_date_time: String,
    subject: String,
    body_preview: String,
    sender: Sender,
    has_attachments: bool,
    internet_message_id: String,
    from: Sender,
    to_recipients: Vec<Sender>,
    cc_recipients: Vec<Sender>,
    bcc_recipients: Vec<Sender>,
    body: Body,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Sender {
    email_address: EmailAddress,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmailAddress {
    name: String,
    address: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Body {
    content_type: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct MailFoldersResponse {
    #[serde(rename = "value")]
    mail_folders: Vec<MailFolder>,
}

impl TryFrom<Message> for EmailResource {
    type Error = anyhow::Error;

    fn try_from(message: Message) -> Result<Self, Self::Error> {
        let email = EmailResource {
            raw_json: serde_json::to_string_pretty(&message)?,
            subject: message.subject,
            from: message.from.email_address.address,
            cc: message
                .cc_recipients
                .into_iter()
                .map(|r| r.email_address.name)
                .collect(),
            bcc: message
                .bcc_recipients
                .into_iter()
                .map(|r| r.email_address.name)
                .collect(),
            references: vec![],
            in_reply_to: None,
            message_id: message.internet_message_id,
            to: message
                .to_recipients
                .into_iter()
                .map(|r| r.email_address.name)
                .collect(),
            date: message.last_modified_date_time,
            text_plain: vec![],
            text_html: vec![message.body.content],
            raw_text: message.body_preview,
            attachments: None,
        };
        Ok(email)
    }
}

pub async fn fetch_emails_from_graph_api(
    token: &AccessToken,
    config: &ImapConfig,
) -> anyhow::Result<HashMap<String, Vec<EmailResource>>> {
    let client = Graph::new(token.bearer_token());

    let res = client
        .me()
        .mail_folders()
        .list_mail_folders()
        .send()
        .await
        .with_context(|| {
            "[ingest_imap]: microsoft_365. Failed to send request to fetch mail folders"
        })?;

    let mail_folders_res: MailFoldersResponse = res
        .json()
        .await
        .with_context(|| "[ingest_imap]: microsoft_365. Deserializing mail folders failed")?;

    let mail_folders = mail_folders_res.mail_folders;

    let fetched_folder_names: Vec<String> = mail_folders
        .clone()
        .into_iter()
        .map(|f| f.display_name.to_lowercase())
        .collect();

    let mut folders_to_be_used = Vec::new();
    for folder_name in &config.folders {
        if !fetched_folder_names.contains(&folder_name.to_lowercase()) {
            warn!(
                "Warning: The folder name '{}' does not exist. The default 'INBOX' will be used.",
                folder_name
            );
        } else {
            folders_to_be_used.push(folder_name)
        }
    }

    let mut emails = HashMap::new();
    for folder in folders_to_be_used {
        let res = client
            .me()
            .mail_folder(folder)
            .messages()
            .list_messages()
            .top(config.max_no_messages.to_string())
            .send().await
            .with_context(|| format!("[ingest_imap]: microsoft_365. Failed to get {:#?} number of emails in the {} folder", config.max_no_messages, folder))?;

        let messages_list: MessageList = res.json().await.with_context(|| {
            "[ingest_imap]: microsoft_365. Deserializing email messages list failed"
        })?;

        let messages = messages_list
            .value
            .into_iter()
            .map(EmailResource::try_from)
            .collect::<anyhow::Result<Vec<EmailResource>>>()?;
          emails.insert(folder.to_string(), messages);
    }

   Ok(emails)
}
