use std::{collections::HashMap, time::Instant};

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use resource_imap::{
    elaboration::{FolderElaboration, ImapElaboration},
    imap, Folder, ImapConfig, ImapResource,
};
use rusqlite::params;
use serde_json::json;
use sha1::{Digest, Sha1};
use tracing::{debug, error};

use crate::{
    cmd::imap::IngestImapArgs,
    ingest::{IngestContext, INS_UR_INGEST_SESSION_FINISH_SQL, INS_UR_INGEST_SESSION_SQL},
};

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
    let mut elaboration = ImapElaboration::new(&config);

    let mut imap_resource = imap(&config).await?;
    imap_resource.init().await?;
    let available_folders = imap_resource.folders().await?;
    let mut folders_to_be_ingested = imap_resource.specified_folders(&config.folder).await?;

    elaboration.discovered_folder_count = available_folders.len();

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

        let start = Instant::now();
        let folder_elaborations = process_folders(
            &mut ingest_stmts,
            &ingest_session_id,
            &device_id,
            &acct_id,
            &mut folders_to_be_ingested,
            &mut imap_resource,
        ).await?;
        let email_ingest_duration = format!("{:.2?}", start.elapsed());


        elaboration.folders = folder_elaborations;
        elaboration.email_ingest_duration = Some(email_ingest_duration);
    }

    match tx.execute(
        INS_UR_INGEST_SESSION_FINISH_SQL,
        params![
            ingest_session_id,
            serde_json::to_string_pretty(&elaboration)?
        ],
    ) {
        Ok(_) => {}
        Err(err) => {
            error!(
                "[ingest_files] unable to execute SQL {} in {}: {}",
                INS_UR_INGEST_SESSION_FINISH_SQL, db_fs_path, err
            )
        }
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

async fn process_folders(
    ingest_stmts: &mut IngestContext<'_>,
    ingest_session_id: &str,
    device_id: &str,
    acct_id: &str,
    folders: &mut [Folder],
    resource: &mut Box<dyn ImapResource>,
) -> Result<HashMap<String, FolderElaboration>> {
    let mut folder_elaborations = HashMap::new();

    for folder in folders.iter_mut() {
        match resource.process_messages_in_folder(folder).await {
            Ok(_) => {}
            Err(err) => {
                error!("{err}");
                continue;
            }
        };

        let Folder {
            name,
            messages,
            metadata,
        } = folder;

        let pb = ProgressBar::new(messages.len() as u64);
        if resource.progress() {
            let pb = ProgressBar::new(messages.len() as u64);
            pb.set_style(ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")?
                .progress_chars("##-"));
            pb.set_message(format!("Processing Messages in folder: {}", name));
        }

        let mut elaboration = FolderElaboration::new(name, messages.len());
        let account_elaboration = json!({ "metadata": serde_json::to_string_pretty(metadata)? });

        let acct_folder_id: String = {
            let start = Instant::now();
            let result = ingest_stmts
                .ur_ingest_session_imap_acct_folder_stmt
                .query_row(
                    params![
                        ingest_session_id,
                        acct_id,
                        name.to_string(),
                        account_elaboration.to_string(),
                    ],
                    |row| row.get(0),
                )?;
            debug!("Account folder ID query time: {:.2?}", start.elapsed());
            result
        };

        let mut text_plain_count = 0;
        let mut html_content_count = 0;

        for email in messages.iter() {
            let text = &email.raw_text;
            let uri = format!("smtp://{}/{}", resource.username(), email.message_id);

            // 4. get all the attachments and do the same

            // 1. insert the raw text into ur, nature is text
            let ur_id: String = {
                let start = Instant::now(); // Start timing
                let result = ingest_stmts.ins_ur_stmt.query_row(
                    params![
                        device_id,
                        ingest_session_id,
                        &None::<String>,
                        format!("smtp://{}/{}", resource.username(), email.message_id),
                        "text".to_string(),
                        email.raw_text,
                        {
                            let mut hasher = Sha1::new();
                            hasher.update(email.raw_text.as_bytes());
                            format!("{:x}", hasher.finalize())
                        },
                        email.raw_text.as_bytes().len(),
                        email.date,
                        &None::<String>, // content_fm_body_attrs
                        &None::<String>, // frontmatter
                        acct_folder_id,
                    ],
                    |row| row.get(0),
                )?;
                debug!("Uniform Resource insert time: {:.2?}", start.elapsed()); // Print elapsed time
                result
            };

            let _ur_sess_message_id: String = {
                let start = Instant::now();
                let result = ingest_stmts
                    .ur_ingest_session_imap_acct_folder_message_stmt
                    .query_row(
                        params![
                            ingest_session_id,
                            acct_folder_id,
                            ur_id,
                            text,
                            email.message_id,
                            email.subject,
                            email.from,
                            serde_json::to_string_pretty(&email.cc).unwrap_or("[]".to_string()),
                            serde_json::to_string_pretty(&email.bcc).unwrap_or("[]".to_string()),
                            serde_json::to_string_pretty(&email.references)
                                .unwrap_or("[".to_string()),
                        ],
                        |row| row.get(0),
                    )?;
                debug!("IMAP Acct Message insert time: {:.2?}", start.elapsed()); // Print elapsed time
                result
            };

            {
                let json = &email.raw_json;
                let size = json.as_bytes().len();
                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(json.as_bytes());
                    format!("{:x}", hasher.finalize())
                };
                let start = Instant::now();
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
                debug!("Full email JSON insert time: {:.2?}", start.elapsed());
            }

            // 3. take out all the text/plain, insert it into ur as a row, nature text
            let start = Instant::now();
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
            debug!(
                "It took {:.2?} to insert {} plain texts in Uniform Resource",
                start.elapsed(),
                email.text_plain.len()
            );
            text_plain_count += email.text_plain.len();

            let start = Instant::now();
            // 4. take out the text/html, insert it into uniform_resource, transform it to json and then put it in uniform_resource_transform.
            for html in &email.text_html {
                let size = html.as_bytes().len();
                let hash = {
                    let mut hasher = Sha1::new();
                    hasher.update(html.as_bytes());
                    format!("{:x}", hasher.finalize())
                };
                let _ur_id: String = ingest_stmts.ins_ur_stmt.query_row(
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
            }
            debug!(
                "It took {:.2?} to insert {} htmls in Uniform Resource",
                start.elapsed(),
                email.text_html.len()
            );
            html_content_count += email.text_html.len();

            if resource.progress() {
                pb.inc(1);
            }
        }

        pb.finish_with_message(format!("Finished processing folder: {}", name));

        // println!(
        //     "Processing all the emails for the {} folder took {:.2?}",
        //     folder.name,
        //     folder_process_start.elapsed()
        // );

        elaboration.html_content_count = html_content_count;
        elaboration.text_plain_count = text_plain_count;
        folder_elaborations.insert(name.to_string(), elaboration);
    }

    Ok(folder_elaborations)
}

fn finalize_transaction(tx: rusqlite::Transaction) -> Result<()> {
    tx.commit()
        .with_context(|| "[ingest_imap] Failed to commit the transaction")
}
