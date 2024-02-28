use std::collections::HashMap;

use anyhow::{Context, Result};
use rusqlite::params;
use sha1::{Digest, Sha1};
use tracing::debug;
use udi_pgp_imap::{process_imap, EmailResource, ImapConfig};

use crate::{
    cmd::imap::IngestImapArgs,
    ingest::{IngestContext, INS_UR_INGEST_SESSION_SQL},
};

use html_parser::Dom;

use super::{upserted_device, DbConn};

/// Main entry point for ingesting emails from IMAP.
pub async fn ingest_imap(args: &IngestImapArgs) -> Result<()> {
    let mut dbc = establish_db_connection(args)?;
    let db_fs_path = &dbc.db_fs_path.clone();

    let tx = start_transaction(&mut dbc, args)?;
    let (device_id, _) = upsert_device(&tx)?;
    let ingest_session_id = create_ingest_session(&tx, &device_id)?;

    debug!("Imap Session: {ingest_session_id}");

    let config: ImapConfig = args.clone().into();
    let email_resources = fetch_email_resources(&config).await?;

    {
        let mut ingest_stmts = IngestContext::from_conn(&tx, db_fs_path)
            .with_context(|| format!("[ingest_imap] ingest_stmts in {}", db_fs_path))?;
        let acct_id: String = ingest_stmts.ur_ingest_session_imap_account_stmt.query_row(
            params![
                ingest_session_id,
                config.username,
                config.password,
                config.addr
            ],
            |row| row.get(0),
        )?;

        process_emails(
            &mut ingest_stmts,
            &ingest_session_id,
            &device_id,
            &acct_id,
            &email_resources,
            &config,
        )?;
    }

    finalize_transaction(tx)
}

/// Establishes a connection to the database.
fn establish_db_connection(args: &IngestImapArgs) -> Result<DbConn> {
    DbConn::new(&args.state_db_fs_path, 0).with_context(|| {
        format!(
            "[ingest_imap] SQLite transaction in {}",
            args.state_db_fs_path
        )
    })
}

fn start_transaction<'a>(
    dbc: &'a mut DbConn,
    args: &'a IngestImapArgs,
) -> Result<rusqlite::Transaction<'a>> {
    dbc.init(Some(&args.state_db_init_sql))
        .with_context(|| "[ingest_imap] Failed to start a database transaction")
}

fn upsert_device(tx: &rusqlite::Transaction) -> Result<(String, String)> {
    upserted_device(tx, &common::DEVICE)
        .with_context(|| format!("[ingest_imap] upserted_device {}", common::DEVICE.name))
}

fn create_ingest_session(tx: &rusqlite::Transaction, device_id: &String) -> Result<String> {
    tx.query_row(
        INS_UR_INGEST_SESSION_SQL,
        params![device_id, None::<String>, None::<String>],
        |row| row.get(0),
    )
    .with_context(|| "[ingest_imap] Failed to create an ingest session")
}

/// Fetches email resources using the IMAP protocol.
async fn fetch_email_resources(config: &ImapConfig) -> Result<HashMap<String, Vec<EmailResource>>> {
    process_imap(config).await.with_context(|| "[ingest_imap] Failed to fetch email resources")
}

/// Processes emails fetched from the IMAP server.
fn process_emails(
    ingest_stmts: &mut IngestContext,
    ingest_session_id: &str,
    device_id: &str,
    acct_id: &str,
    email_resources: &HashMap<String, Vec<EmailResource>>,
    config: &ImapConfig,
) -> Result<()> {
    for (folder, emails) in email_resources {
        // insert folder into
        let acct_folder_id: String = ingest_stmts
            .ur_ingest_session_imap_acct_folder_stmt
            .query_row(params![ingest_session_id, acct_id, folder], |row| {
                row.get(0)
            })?;

        for email in emails.iter() {
            let text = &email.raw_text;
            let size = text.as_bytes().len();
            let hash = {
                let mut hasher = Sha1::new();
                hasher.update(text.as_bytes());
                format!("{:x}", hasher.finalize())
            };
            let uri = format!("smtp://{}/{}", config.username.clone().unwrap(), email.message_id);

            // 4. get all the attachments and do the same

            // 1. insert the raw text into ur, nature is text
            let ur_id: String = ingest_stmts.ins_ur_stmt.query_row(
                params![
                    device_id,
                    ingest_session_id,
                    &None::<String>,
                    uri,
                    "text".to_string(),
                    email.raw_text,
                    hash,
                    size,
                    email.date,
                    &None::<String>, // content_fm_body_attrs
                    &None::<String>, // frontmatter
                    acct_folder_id,
                ],
                |row| row.get(0),
            )?;

            let _ur_sess_message_id: String = ingest_stmts
                .ur_ingest_session_imap_acct_folder_message_stmt
                .query_row(
                    params![
                        ingest_session_id,
                        acct_folder_id,
                        ur_id,
                        text,
                        email.message_id
                    ],
                    |row| row.get(0),
                )?;

            {
                let json = &email.raw_json;
                let size = json.as_bytes().len();
                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(json.as_bytes());
                    format!("{:x}", hasher.finalize())
                };

                // 2. insert the whole json into ur, nature is json
                let _: String = ingest_stmts.ins_ur_stmt.query_row(
                    params![
                        device_id,
                        ingest_session_id,
                        &None::<String>,
                        format!("{uri}/json"),
                        "json".to_string(),
                        json,
                        hash,
                        size,
                        email.date,
                        &None::<String>, // content_fm_body_attrs
                        &None::<String>, // frontmatter
                        acct_folder_id,
                    ],
                    |row| row.get(0),
                )?;
            }

            // 3. take out all the text/plain, insert it into ur as a row, nature text
            for plain_text in &email.text_plain {
                let size = plain_text.as_bytes().len();
                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(plain_text.as_bytes());
                    format!("{:x}", hasher.finalize())
                };

                let _: String = ingest_stmts.ins_ur_stmt.query_row(
                    params![
                        device_id,
                        ingest_session_id,
                        &None::<String>,
                        format!("{uri}/txt"),
                        "txt".to_string(),
                        plain_text,
                        hash,
                        size,
                        email.date,
                        &None::<String>, // content_fm_body_attrs
                        &None::<String>, // frontmatter
                        acct_folder_id,
                    ],
                    |row| row.get(0),
                )?;
            }

            // 4. take out the text/html, insert it into uniform_resource, transform it to json and then put it in uniform_resource_transform.
            for html in &email.text_html {
                let size = html.as_bytes().len();
                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(html.as_bytes());
                    format!("{:x}", hasher.finalize())
                };
                let ur_id: String = ingest_stmts.ins_ur_stmt.query_row(
                    params![
                        device_id,
                        ingest_session_id,
                        &None::<String>,
                        format!("{uri}/html"),
                        "html".to_string(),
                        html,
                        hash,
                        size,
                        email.date,
                        &None::<String>, // content_fm_body_attrs
                        &None::<String>, // frontmatter
                        acct_folder_id,
                    ],
                    |row| row.get(0),
                )?;

                let html = clean_html(html);

                let html_json = Dom::parse(html)?.to_json_pretty()?;

                let html_json_size = html_json.as_bytes().len();
                let html_json_hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(html_json.as_bytes());
                    format!("{:x}", hasher.finalize())
                };

                let _ur_transform_id: String = ingest_stmts.ins_ur_transform_stmt.query_row(
                    params![
                        ur_id,
                        format!("{uri}/json"),
                        "json",
                        html_json_hash,
                        html_json,
                        html_json_size
                    ],
                    |row| row.get(0),
                )?;
            }
        }
    }
    Ok(())
}

fn finalize_transaction(tx: rusqlite::Transaction) -> Result<()> {
    tx.commit()
        .with_context(|| "[ingest_imap] Failed to commit the transaction")
}

// I found that some of the html strings don't start with <!DOCTYPE> which breaks
// html to json parsing
fn clean_html(html: &str) -> &str {
    if let Some(index) = html.to_lowercase().find("<!doctype") {
        if index > 0 {
            &html[index..]
        } else {
            html
        }
    } else {
        html
    }
}