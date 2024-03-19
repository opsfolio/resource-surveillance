use crate::{EmailResource, Folder, ImapConfig};
use anyhow::Context;
use graph_rs_sdk::{oauth::AccessToken, Graph, ODataQuery};
use serde::{Deserialize, Serialize};

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
    config: &mut ImapConfig,
) -> anyhow::Result<Vec<Folder>> {
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

    let folders = mail_folders_res
        .mail_folders
        .clone()
        .into_iter()
        // deleted items has a totally different schema
        .filter(|m| {
            m.total_item_count > 0
                && m.display_name != "Deleted Items"
                && (config.folder == "*"
                    || m.display_name
                        .to_lowercase()
                        .contains(&config.folder.to_lowercase()))
        })
        .collect::<Vec<_>>();

    let mut emails = Vec::new();
    for folder in &folders {
        config.mailboxes.push(folder.display_name.to_string());

        let folder_name = folder.display_name.replace(' ', "");
        let mut all_messages = Vec::new();
        let mut skip_count = 0;

        loop {
            let batch_size = std::cmp::min(config.batch_size, 1000);
            let res = client
                .me()
                .mail_folder(&folder_name)
                .messages()
                .list_messages()
                .top(batch_size.to_string()) // limit the no of emails to 1000
                .skip(skip_count.to_string()) // offset
                .send()
                .await
                .with_context(|| {
                    format!(
                        "[ingest_imap]: microsoft_365. Failed to get emails in the {} folder",
                        folder_name
                    )
                })?;

            let messages_list: MessageList = res.json().await.with_context(|| {
                "[ingest_imap]: microsoft_365. Deserializing email messages list failed"
            })?;

            if messages_list.value.is_empty() {
                break;
            }

            let messages: Vec<EmailResource> = messages_list
                .value
                .into_iter()
                .map(EmailResource::try_from)
                .collect::<anyhow::Result<Vec<EmailResource>>>()?;

            all_messages.extend(messages);

            // Update skip_count for the next batch
            skip_count += batch_size;
        }
        emails.push(Folder {
            name: folder_name.to_string(),
            metadata: serde_json::to_value(folder)?,
            messages: all_messages,
        });
    }

    Ok(emails)
}
