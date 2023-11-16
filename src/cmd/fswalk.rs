use anyhow::{Context, Result};
use indoc::indoc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_regex;

use crate::fsresource::*;
use crate::persist::*;
use crate::resource::*;

pub struct UniformResourceWriterState<'a> {
    device_id: &'a String,
    walk_session_id: &'a String,
    walk_path_id: &'a String,
}

// TODO: switch the actual URI (first parameter) to an &str? to save memory?
#[derive(Debug)]
pub enum UniformResourceWriterAction {
    Inserted(String),
    ContentSupplierError(Box<dyn std::error::Error>),
    ContentUnavailable(),
    Error(anyhow::Error),
}

#[derive(Debug)]
pub struct UniformResourceWriterResult {
    uri: String,
    action: UniformResourceWriterAction,
}

pub trait UniformResourceWriter<Resource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult;

    fn insert_text(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
        resource: &ContentResource,
    ) -> UniformResourceWriterResult {
        let uri = resource.uri.clone();
        match resource.content_text_supplier.as_ref() {
            Some(text_supplier) => match text_supplier() {
                Ok(text) => match ins_ur_stmt.query_row(
                    params![
                        urw_state.device_id,
                        urw_state.walk_session_id,
                        urw_state.walk_path_id,
                        resource.uri,
                        resource.nature,
                        text.content_text(),
                        text.content_digest_hash(),
                        resource.size,
                        resource.last_modified_at.unwrap().to_string(),
                        &None::<String>,
                        &None::<String>
                    ],
                    |row| row.get(0),
                ) {
                    Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                        uri,
                        action: UniformResourceWriterAction::Inserted(new_or_existing_ur_id),
                    },
                    Err(err) => UniformResourceWriterResult {
                        uri,
                        action: UniformResourceWriterAction::Error(err.into()),
                    },
                },
                Err(err) => UniformResourceWriterResult {
                    uri,
                    action: UniformResourceWriterAction::ContentSupplierError(err),
                },
            },
            None => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::ContentUnavailable(),
            },
        }
    }

    fn insert_binary(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
        resource: &ContentResource,
        bc: Box<dyn BinaryContent>,
    ) -> UniformResourceWriterResult {
        let uri = resource.uri.clone();
        match ins_ur_stmt.query_row(
            params![
                urw_state.device_id,
                urw_state.walk_session_id,
                urw_state.walk_path_id,
                resource.uri,
                resource.nature,
                bc.content_binary(),
                bc.content_digest_hash(),
                resource.size,
                resource.last_modified_at.unwrap().to_string(),
                &None::<String>,
                &None::<String>
            ],
            |row| row.get(0),
        ) {
            Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Inserted(new_or_existing_ur_id),
            },
            Err(err) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Error(err.into()),
            },
        }
    }
}

// this is the unknown resource content handler
impl UniformResourceWriter<ContentResource> for ContentResource {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        let uri = self.uri.clone();
        match ins_ur_stmt.query_row(
            params![
                urw_state.device_id,
                urw_state.walk_session_id,
                urw_state.walk_path_id,
                self.uri,
                self.nature,
                &None::<String>,
                String::from("-"),
                self.size,
                self.last_modified_at.unwrap().to_string(),
                &None::<String>,
                &None::<String>
            ],
            |row| row.get(0),
        ) {
            Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Inserted(new_or_existing_ur_id),
            },
            Err(err) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Error(err.into()),
            },
        }
    }
}

impl UniformResourceWriter<ContentResource> for HtmlResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

impl UniformResourceWriter<ContentResource> for ImageResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        let uri = self.resource.uri.clone();
        match self.resource.content_binary_supplier.as_ref() {
            Some(image_supplier) => match image_supplier() {
                Ok(image_src) => {
                    self.insert_binary(ins_ur_stmt, urw_state, &self.resource, image_src)
                }
                Err(err) => UniformResourceWriterResult {
                    uri,
                    action: UniformResourceWriterAction::ContentSupplierError(err),
                },
            },
            None => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::ContentUnavailable(),
            },
        }
    }
}

impl UniformResourceWriter<ContentResource> for JsonResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

impl UniformResourceWriter<ContentResource> for MarkdownResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        let uri = self.resource.uri.clone();
        match self.resource.content_text_supplier.as_ref() {
            Some(md_supplier) => match md_supplier() {
                Ok(markdown_src) => {
                    let mut fm_attrs = None::<String>;
                    let mut fm_json: Option<String> = None::<String>;
                    let (_, fm_raw, fm_json_value, fm_body) = markdown_src.frontmatter();
                    if fm_json_value.is_ok() {
                        fm_json = Some(serde_json::to_string_pretty(&fm_json_value.ok()).unwrap());
                        let fm_attrs_value = serde_json::json!({
                            "frontMatter": fm_raw.unwrap(),
                            "body": fm_body,
                            "attrs": fm_json.clone().unwrap()
                        });
                        fm_attrs = Some(serde_json::to_string_pretty(&fm_attrs_value).unwrap());
                    }
                    let uri = self.resource.uri.to_string();
                    match ins_ur_stmt.query_row(
                        params![
                            urw_state.device_id,
                            urw_state.walk_session_id,
                            urw_state.walk_path_id,
                            self.resource.uri,
                            self.resource.nature,
                            markdown_src.content_text(),
                            markdown_src.content_digest_hash(),
                            self.resource.size,
                            self.resource.last_modified_at.unwrap().to_string(),
                            fm_attrs,
                            fm_json
                        ],
                        |row| row.get(0),
                    ) {
                        Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                            uri,
                            action: UniformResourceWriterAction::Inserted(new_or_existing_ur_id),
                        },
                        Err(err) => UniformResourceWriterResult {
                            uri,
                            action: UniformResourceWriterAction::Error(err.into()),
                        },
                    }
                }
                Err(err) => UniformResourceWriterResult {
                    uri,
                    action: UniformResourceWriterAction::ContentSupplierError(err),
                },
            },
            None => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::ContentUnavailable(),
            },
        }
    }
}

impl UniformResourceWriter<ContentResource> for SoftwarePackageDxResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

impl UniformResourceWriter<ContentResource> for SvgResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

impl UniformResourceWriter<ContentResource> for TestAnythingResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

impl UniformResourceWriter<ContentResource> for TomlResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

impl UniformResourceWriter<ContentResource> for YamlResource<ContentResource> {
    fn insert(
        &self,
        ins_ur_stmt: &mut rusqlite::Statement<'_>,
        urw_state: &UniformResourceWriterState<'_>,
    ) -> UniformResourceWriterResult {
        self.insert_text(ins_ur_stmt, urw_state, &self.resource)
    }
}

// TODO: Allow per file type / MIME / extension / nature configuration
// such as compression for certain types of files but as-is for other
// types;

#[derive(Serialize, Deserialize)]
pub struct FsWalkBehavior {
    pub root_paths: Vec<String>,

    #[serde(with = "serde_regex")]
    pub ignore_entries: Vec<regex::Regex>,

    #[serde(with = "serde_regex")]
    pub ingest_content: Vec<regex::Regex>,

    #[serde(with = "serde_regex")]
    pub compute_digests: Vec<regex::Regex>,
}

impl FsWalkBehavior {
    pub fn new(
        device_id: &String,
        fsw_args: &super::FsWalkArgs,
        conn: &Connection,
    ) -> anyhow::Result<(Self, Option<String>)> {
        if let Some(behavior_name) = &fsw_args.behavior {
            let (fswb_behavior_id, fswb_json): (String, String) = conn
                .query_row(
                    r#"
                   SELECT behavior_id, behavior_conf_json 
                     FROM behavior 
                    WHERE device_id = ?1 AND behavior_name = ?2 
                 ORDER BY created_at desc 
                    LIMIT 1"#,
                    params![device_id, behavior_name],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .with_context(|| {
                    format!(
                        "[fs_walk] unable to read behavior '{}' from {} behavior table",
                        behavior_name, fsw_args.state_db_fs_path
                    )
                })?;
            let fswb = FsWalkBehavior::from_json(&fswb_json).with_context(|| {
                format!(
                    "[fs_walk] unable to deserialize behavior {} in {}",
                    fswb_json, fsw_args.state_db_fs_path
                )
            })?;
            Ok((fswb, Some(fswb_behavior_id)))
        } else {
            Ok((FsWalkBehavior::from_fs_walk_args(fsw_args), None))
        }
    }

    pub fn from_fs_walk_args(args: &super::FsWalkArgs) -> Self {
        FsWalkBehavior {
            root_paths: args.root_path.clone(),
            ingest_content: args.surveil_content.clone(),
            compute_digests: args.compute_digests.clone(),
            ignore_entries: args.ignore_entry.clone(),
        }
    }

    pub fn from_json(json_text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_text)
    }

    pub fn persistable_json_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn save(
        &self,
        conn: &Connection,
        device_id: &String,
        behavior_name: &String,
    ) -> anyhow::Result<String> {
        let behavior_id: String = conn
            .query_row(
                r#"
             INSERT INTO behavior (behavior_id, device_id, behavior_name, behavior_conf_json)
                           VALUES (ulid(), ?, ?, ?)
             ON CONFLICT (device_id, behavior_name) DO UPDATE
                     SET behavior_conf_json = EXCLUDED.behavior_conf_json, 
                         updated_at = CURRENT_TIMESTAMP
               RETURNING behavior_id"#,
                params![
                    device_id,
                    behavior_name,
                    self.persistable_json_text().unwrap() // TODO: do proper error checking, don't panic
                ],
                |row| row.get(0),
            )
            .with_context(|| format!("[fs_walk] unable to save behavior '{}'", behavior_name))?;
        Ok(behavior_id)
    }
}

pub fn fs_walk(cli: &super::Cli, fsw_args: &super::FsWalkArgs) -> Result<String> {
    let db_fs_path = &fsw_args.state_db_fs_path;

    if cli.debug == 1 {
        println!("Surveillance State DB: {}", db_fs_path);
    }

    let mut conn = Connection::open(db_fs_path)
        .with_context(|| format!("[fs_walk] SQLite database {}", db_fs_path))?;

    prepare_conn(&conn)
        .with_context(|| format!("[fs_walk] prepare SQLite connection for {}", db_fs_path))?;

    // putting everything inside a transaction improves performance significantly
    let tx = conn
        .transaction()
        .with_context(|| format!("[fs_walk] SQLite transaction in {}", db_fs_path))?;

    execute_migrations(&tx, "fs_walk")
        .with_context(|| format!("[fs_walk] execute_migrations in {}", db_fs_path))?;

    // insert the device or, if it exists, get its current ID and name
    let (device_id, device_name) = upserted_device(&tx, &crate::DEVICE).with_context(|| {
        format!(
            "[fs_walk] upserted_device {} in {}",
            crate::DEVICE.name,
            db_fs_path
        )
    })?;

    if cli.debug == 1 {
        println!("Device: {device_name} ({device_id})");
    }

    let mut ignore_db_fs_path: Vec<String> = Vec::new();
    if !fsw_args.include_state_db_in_walk {
        let canonical_db_fs_path = std::fs::canonicalize(std::path::Path::new(&db_fs_path))
            .with_context(|| format!("[fs_walk] unable to canonicalize in {}", db_fs_path))?;
        let canonical_db_fs_path = canonical_db_fs_path.to_string_lossy().to_string();
        let mut wal_path = std::path::PathBuf::from(&canonical_db_fs_path);
        let mut db_journal_path = std::path::PathBuf::from(&canonical_db_fs_path);
        wal_path.set_extension("wal");
        db_journal_path.set_extension("db-journal");
        ignore_db_fs_path.push(canonical_db_fs_path);
        ignore_db_fs_path.push(wal_path.to_string_lossy().to_string());
        ignore_db_fs_path.push(db_journal_path.to_string_lossy().to_string());
    }

    // the ulid() function we're using below is not built into SQLite, we define
    // it in persist::prepare_conn.

    // separate the SQL from the execute so we can use it in logging, errors, etc.
    const INS_UR_WALK_SESSION_SQL: &str = indoc! {"
        INSERT INTO ur_walk_session (ur_walk_session_id, device_id, behavior_id, behavior_json, walk_started_at) 
                             VALUES (ulid(), ?, ?, ?, CURRENT_TIMESTAMP) RETURNING ur_walk_session_id"};
    const INS_UR_WALK_SESSION_FINISH_SQL: &str = indoc! {"
        UPDATE ur_walk_session 
           SET walk_finished_at = CURRENT_TIMESTAMP 
         WHERE ur_walk_session_id = ?"};
    const INS_UR_WSP_SQL: &str = indoc! {"
        INSERT INTO ur_walk_session_path (ur_walk_session_path_id, walk_session_id, root_path) 
                                  VALUES (ulid(), ?, ?) RETURNING ur_walk_session_path_id"};
    // in ins_ur_stmt the `DO UPDATE SET size_bytes = EXCLUDED.size_bytes` is a workaround to force uniform_resource_id when the row already exists
    const INS_UR_SQL: &str = indoc! {"
        INSERT INTO uniform_resource (uniform_resource_id, device_id, walk_session_id, walk_path_id, uri, nature, content, content_digest, size_bytes, last_modified_at, content_fm_body_attrs, frontmatter)
                              VALUES (ulid(), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
                         ON CONFLICT (device_id, content_digest, uri, size_bytes, last_modified_at) 
                           DO UPDATE SET size_bytes = EXCLUDED.size_bytes
                           RETURNING uniform_resource_id"};
    const INS_UR_FS_ENTRY_SQL: &str = indoc! {"
        INSERT INTO ur_walk_session_path_fs_entry (ur_walk_session_path_fs_entry_id, walk_session_id, walk_path_id, uniform_resource_id, file_path_abs, file_path_rel_parent, file_path_rel, file_basename, file_extn, ur_status, ur_diagnostics) 
                                           VALUES (ulid(), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"};

    let (fswb, mut behavior_id) = FsWalkBehavior::new(&device_id, fsw_args, &tx)
        .with_context(|| format!("[fs_walk] behavior issue {}", db_fs_path))?;
    if let Some(save_behavior_name) = &fsw_args.save_behavior {
        let saved_bid = fswb
            .save(&tx, &device_id, save_behavior_name)
            .with_context(|| {
                format!("[fs_walk] saving {} in {}", save_behavior_name, db_fs_path)
            })?;
        if cli.debug == 1 {
            println!("Saved behavior: {} ({})", save_behavior_name, saved_bid);
        }
        behavior_id = Some(saved_bid);
    }
    if cli.debug == 1 {
        println!(
            "Behavior: {}",
            behavior_id.clone().unwrap_or(String::from("custom"))
        );
    }

    let walk_session_id: String = tx
        .query_row(
            INS_UR_WALK_SESSION_SQL,
            params![
                device_id,
                behavior_id,
                match fswb.persistable_json_text() {
                    Ok(json_text) => json_text,
                    Err(_err) =>
                        String::from("JSON serialization error, TODO: convert err to string"),
                }
            ],
            |row| row.get(0),
        )
        .with_context(|| {
            format!(
                "[fs_walk] inserting UR walk session using {} in {}",
                INS_UR_WALK_SESSION_SQL, db_fs_path
            )
        })?;
    if cli.debug == 1 {
        println!("Walk Session: {walk_session_id}");
    }

    // TODO: https://github.com/opsfolio/resource-surveillance/issues/16
    //       from this point on, since we have a walk session put all errors
    //       into an error log table inside the database associated with each
    //       session (e.g. ur_walk_session_telemetry) and only report to CLI if
    //       writing the log into the database fails.

    // Use a scope to ensure all prepared SQL statements, which borrow `tx`` are dropped before committing the transaction.
    {
        let mut ins_ur_wsp_stmt = tx.prepare(INS_UR_WSP_SQL).with_context(|| {
            format!(
                "[fs_walk] unable to create `ins_ur_wsp_stmt` SQL {} in {}",
                INS_UR_WSP_SQL, db_fs_path
            )
        })?;
        let mut ins_ur_stmt = tx.prepare(INS_UR_SQL).with_context(|| {
            format!(
                "[fs_walk] unable to create `ins_ur_stmt` SQL {} in {}",
                INS_UR_SQL, db_fs_path
            )
        })?;
        let mut ins_ur_fs_entry_stmt = tx.prepare(INS_UR_FS_ENTRY_SQL).with_context(|| {
            format!(
                "[fs_walk] unable to create `ins_ur_fs_entry_stmt` SQL {} in {}",
                INS_UR_FS_ENTRY_SQL, db_fs_path
            )
        })?;

        for root_path in &fswb.root_paths {
            let canonical_path_buf = std::fs::canonicalize(std::path::Path::new(&root_path))
                .with_context(|| {
                    format!(
                        "[fs_walk] unable to canonicalize {} in {}",
                        root_path, db_fs_path
                    )
                })?;
            let canonical_path = canonical_path_buf.into_os_string().into_string().unwrap();

            let ins_ur_wsp_params = params![walk_session_id, canonical_path];
            let walk_path_id: String = ins_ur_wsp_stmt
                .query_row(ins_ur_wsp_params, |row| row.get(0))
                .with_context(|| {
                    format!(
                        "[fs_walk] ins_ur_wsp_stmt {} with {} in {}",
                        INS_UR_WSP_SQL, "TODO: ins_ur_wsp_params.join()", db_fs_path
                    )
                })?;
            if cli.debug == 1 {
                println!("  Walk Session Path: {root_path} ({walk_path_id})");
            }

            let urw_state = UniformResourceWriterState {
                device_id: &device_id,
                walk_session_id: &walk_session_id,
                walk_path_id: &walk_path_id,
            };

            let rp: Vec<String> = vec![canonical_path.clone()];
            let walker =
                FileSysResourcesWalker::new(&rp, &fswb.ignore_entries, &fswb.ingest_content)
                    .with_context(|| {
                        format!(
                            "[fs_walk] unable to walker for {} in {}",
                            canonical_path, db_fs_path
                        )
                    })?;

            for resource_result in walker.walk_resources_iter() {
                match resource_result {
                    Ok(resource) => {
                        let inserted = match resource {
                            UniformResource::Html(html) => {
                                html.insert(&mut ins_ur_stmt, &urw_state)
                            }
                            UniformResource::Json(json) => {
                                json.insert(&mut ins_ur_stmt, &urw_state)
                            }
                            UniformResource::Image(img) => img.insert(&mut ins_ur_stmt, &urw_state),
                            UniformResource::Markdown(md) => {
                                md.insert(&mut ins_ur_stmt, &urw_state)
                            }
                            UniformResource::SpdxJson(spdx) => {
                                spdx.insert(&mut ins_ur_stmt, &urw_state)
                            }
                            UniformResource::Svg(svg) => svg.insert(&mut ins_ur_stmt, &urw_state),
                            UniformResource::Tap(tap) => tap.insert(&mut ins_ur_stmt, &urw_state),
                            UniformResource::Toml(toml) => {
                                toml.insert(&mut ins_ur_stmt, &urw_state)
                            }
                            UniformResource::Yaml(yaml) => {
                                yaml.insert(&mut ins_ur_stmt, &urw_state)
                            }
                            UniformResource::Unknown(unknown) => {
                                unknown.insert(&mut ins_ur_stmt, &urw_state)
                            }
                        };

                        let uniform_resource_id = match inserted.action {
                            UniformResourceWriterAction::Inserted(ref uniform_resource_id) => {
                                Some(uniform_resource_id)
                            }
                            _ => None,
                        };

                        // don't store the database we're creating in the walk unless requested
                        if !fsw_args.include_state_db_in_walk
                            && ignore_db_fs_path.iter().any(|s| s == &inserted.uri)
                        {
                            continue;
                        }

                        match extract_path_info(
                            std::path::Path::new(&canonical_path),
                            std::path::Path::new(&inserted.uri),
                        ) {
                            Some((
                                file_path_abs,
                                file_path_rel_parent,
                                file_path_rel,
                                file_basename,
                                file_extn,
                            )) => {
                                match ins_ur_fs_entry_stmt.execute(params![
                                    walk_session_id,
                                    walk_path_id,
                                    uniform_resource_id,
                                    file_path_abs.into_os_string().into_string().unwrap(),
                                    file_path_rel_parent.into_os_string().into_string().unwrap(),
                                    file_path_rel.into_os_string().into_string().unwrap(),
                                    file_basename,
                                    if let Some(file_extn) = file_extn {
                                        file_extn
                                    } else {
                                        String::from("")
                                    },
                                    match inserted.action {
                                        UniformResourceWriterAction::Inserted(_) => None,
                                        UniformResourceWriterAction::ContentSupplierError(_) | UniformResourceWriterAction::Error(_) =>
                                            Some(String::from("ERROR")),
                                        UniformResourceWriterAction::ContentUnavailable() =>
                                            Some(String::from("ISSUE")),
                                    },
                                    match inserted.action {
                                        UniformResourceWriterAction::Inserted(_) => None,
                                        UniformResourceWriterAction::ContentSupplierError(_) =>
                                            Some(String::from(
                                                r#"{ "error": "TODO: serialize content supplier error" }"#
                                            )),
                                        UniformResourceWriterAction::ContentUnavailable() =>
                                            Some(String::from(
                                                r#"{ "issue": "content supplier was not provided for", "remediation": "see CLI args/config and request content for this extension" }"#
                                            )),
                                        UniformResourceWriterAction::Error(_) => Some(
                                            String::from(r#"{ "error": "TODO: serialize error" }"#)
                                        ),
                                    }
                                ]) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        eprintln!( "[fs_walk] unable to insert UR walk session path file system entry for {} in {}: {} ({})",
                                        &inserted.uri, db_fs_path, err, INS_UR_FS_ENTRY_SQL
                                        )
                                    }
                                }
                            }
                            None => {
                                eprintln!(
                                    "[fs_walk] error extracting path info for {} in {}",
                                    canonical_path, db_fs_path
                                )
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error processing a resource: {}", e);
                    }
                }
            }
        }
    }

    match tx.execute(INS_UR_WALK_SESSION_FINISH_SQL, params![walk_session_id]) {
        Ok(_) => {}
        Err(err) => {
            eprintln!(
                "[fs_walk] unable to execute SQL {} in {}: {}",
                INS_UR_WALK_SESSION_FINISH_SQL, db_fs_path, err
            )
        }
    }
    // putting everything inside a transaction improves performance significantly
    tx.commit()
        .with_context(|| format!("[fs_walk] unable to perform final commit in {}", db_fs_path))?;

    Ok(walk_session_id)
}
