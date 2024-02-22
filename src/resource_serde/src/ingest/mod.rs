use crate::cmd::IngestFilesArgs;
use anyhow::{Context, Result};
use autometrics::autometrics;
use indoc::indoc;
use resource::shell::ShellResult;
use resource::shell::ShellStdIn;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::persist::*;
use resource::*;

mod files;
mod imap;
mod tasks;

pub use files::ingest_files;
pub use imap::ingest_imap;
pub use tasks::ingest_tasks;

// separate the SQL from the execute so we can use it in logging, errors, etc.
const INS_UR_INGEST_SESSION_SQL: &str = indoc! {"
        INSERT INTO ur_ingest_session (ur_ingest_session_id, device_id, behavior_id, behavior_json, ingest_started_at) 
                             VALUES (ulid(), ?, ?, ?, CURRENT_TIMESTAMP) RETURNING ur_ingest_session_id"};

const INS_UR_INGEST_SESSION_FINISH_SQL: &str = indoc! {"
        UPDATE ur_ingest_session 
           SET ingest_finished_at = CURRENT_TIMESTAMP 
         WHERE ur_ingest_session_id = ?"};

const INS_UR_ISFSP_SQL: &str = indoc! {"
        INSERT INTO ur_ingest_session_fs_path (ur_ingest_session_fs_path_id, ingest_session_id, root_path) 
                                  VALUES (ulid(), ?, ?) RETURNING ur_ingest_session_fs_path_id"};

// in INS_UR_SQL the `DO UPDATE SET size_bytes = EXCLUDED.size_bytes` is a workaround to allow RETURNING uniform_resource_id when the row already exists
const INS_UR_SQL: &str = indoc! {"
        INSERT INTO uniform_resource (uniform_resource_id, device_id, ingest_session_id, ingest_fs_path_id, uri, nature, content, content_digest, size_bytes, last_modified_at, content_fm_body_attrs, frontmatter)
                              VALUES (ulid(), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
                         ON CONFLICT (device_id, content_digest, uri, size_bytes, last_modified_at) 
                           DO UPDATE SET size_bytes = EXCLUDED.size_bytes
                           RETURNING uniform_resource_id"};

const INS_UR_TRANSFORM_SQL: &str = indoc! {"
        INSERT INTO uniform_resource_transform (uniform_resource_transform_id, uniform_resource_id, uri, nature, content_digest, content, size_bytes)
                                        VALUES (ulid(), ?, ?, ?, ?, ?, ?) 
                                   ON CONFLICT (uniform_resource_id, content_digest, nature, size_bytes) 
                                 DO UPDATE SET size_bytes = EXCLUDED.size_bytes
                                     RETURNING uniform_resource_transform_id"};

const INS_UR_ISFSP_ENTRY_SQL: &str = indoc! {"
        INSERT INTO ur_ingest_session_fs_path_entry (ur_ingest_session_fs_path_entry_id, ingest_session_id, ingest_fs_path_id, uniform_resource_id, file_path_abs, file_path_rel_parent, file_path_rel, file_basename, file_extn, ur_status, ur_diagnostics, captured_executable) 
                                           VALUES (ulid(), ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"};

const INS_UR_IS_TASK_SQL: &str = indoc! {"
        INSERT INTO ur_ingest_session_task (ur_ingest_session_task_id, ingest_session_id, uniform_resource_id, captured_executable, ur_status, ur_diagnostics) 
                                            VALUES (ulid(), ?, ?, ?, ?, ?)"};

const INS_UR_INGEST_SESSION_IMAP_ACCT: &str = indoc! {"
INSERT INTO ur_ingest_session_imap_account (ur_ingest_session_imap_account_id, ingest_session_id, email, password, host, elaboration, created_at, created_by) 
VALUES (ulid(), ?, ?, ?, ?, '{}', CURRENT_TIMESTAMP, 'system') 
ON CONFLICT (ingest_session_id, email) 
DO UPDATE SET password = EXCLUDED.password, host = EXCLUDED.host 
RETURNING ur_ingest_session_imap_account_id;"};

const INS_UR_INGEST_SESSION_IMAP_ACCT_FOLDER: &str = indoc! {"INSERT INTO ur_ingest_session_imap_acct_folder (ur_ingest_session_imap_acct_folder_id, ingest_session_id, ingest_account_id, folder_name, elaboration, created_at, created_by)
VALUES (ulid(), ?, ?, ?, '{}', CURRENT_TIMESTAMP, 'system') 
ON CONFLICT (ingest_account_id, folder_name) 
DO UPDATE SET created_at = EXCLUDED.created_at RETURNING ur_ingest_session_imap_acct_folder_id;"};

const INS_UR_INGEST_SESSION_IMAP_ACCT_FOLDER_MESSAGE: &str = indoc! {"
INSERT INTO ur_ingest_session_imap_acct_folder_message (ur_ingest_session_imap_acct_folder_message_id, ingest_session_id, ingest_imap_acct_folder_id, uniform_resource_id, message, created_at, created_by)
VALUES (ulid(), ?, ?, ?, ?, CURRENT_TIMESTAMP, 'system') RETURNING ur_ingest_session_imap_acct_folder_message_id;"};

#[allow(dead_code)]
#[derive(Debug)]
pub struct IngestContext<'conn> {
    ins_ur_isfsp_stmt: rusqlite::Statement<'conn>,
    ins_ur_stmt: rusqlite::Statement<'conn>,
    ins_ur_transform_stmt: rusqlite::Statement<'conn>,
    ins_ur_isfsp_entry_stmt: rusqlite::Statement<'conn>,
    ins_ur_is_task_stmt: rusqlite::Statement<'conn>,
    ur_ingest_session_imap_account_stmt: rusqlite::Statement<'conn>,
    ur_ingest_session_imap_acct_folder_stmt: rusqlite::Statement<'conn>,
    ur_ingest_session_imap_acct_folder_message_stmt: rusqlite::Statement<'conn>,
}

impl<'conn> IngestContext<'conn> {
    #[autometrics]
    pub fn from_conn(conn: &'conn Connection, db_fs_path: &str) -> Result<IngestContext<'conn>> {
        let ins_ur_isfsp_stmt = conn.prepare(INS_UR_ISFSP_SQL).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ins_ur_isfsp_stmt` SQL {} in {}",
                INS_UR_ISFSP_SQL, db_fs_path
            )
        })?;
        let ins_ur_stmt = conn.prepare(INS_UR_SQL).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ins_ur_stmt` SQL {} in {}",
                INS_UR_SQL, db_fs_path
            )
        })?;
        let ins_ur_transform_stmt = conn.prepare(INS_UR_TRANSFORM_SQL).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ins_ur_transform_stmt` SQL {} in {}",
                INS_UR_TRANSFORM_SQL, db_fs_path
            )
        })?;
        let ins_ur_isfsp_entry_stmt = conn.prepare(INS_UR_ISFSP_ENTRY_SQL).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ins_ur_isfsp_entry_stmt` SQL {} in {}",
                INS_UR_ISFSP_ENTRY_SQL, db_fs_path
            )
        })?;
        let ins_ur_istask_entry_stmt = conn.prepare(INS_UR_IS_TASK_SQL).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ins_ur_istask_entry_stmt` SQL {} in {}",
                INS_UR_ISFSP_ENTRY_SQL, db_fs_path
            )
        })?;

        let ur_ingest_session_imap_account_stmt = conn.prepare(INS_UR_INGEST_SESSION_IMAP_ACCT).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ur_ingest_session_imap_account_stmt` SQL {} in {}",
                INS_UR_ISFSP_ENTRY_SQL, db_fs_path
            )
        })?;

        let ur_ingest_session_imap_acct_folder_stmt = conn.prepare(INS_UR_INGEST_SESSION_IMAP_ACCT_FOLDER).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ur_ingest_session_imap_acct_folder_stmt` SQL {} in {}",
                INS_UR_ISFSP_ENTRY_SQL, db_fs_path
            )
        })?;

        let ur_ingest_session_imap_acct_folder_message_stmt = conn.prepare(INS_UR_INGEST_SESSION_IMAP_ACCT_FOLDER_MESSAGE).with_context(|| {
            format!(
                "[IngestContext::from_conn] unable to create `ur_ingest_session_imap_acct_folder_message_stmt` SQL {} in {}",
                INS_UR_ISFSP_ENTRY_SQL, db_fs_path
            )
        })?;


        Ok(IngestContext {
            ins_ur_isfsp_stmt,
            ins_ur_stmt,
            ins_ur_transform_stmt,
            ins_ur_isfsp_entry_stmt,
            ins_ur_is_task_stmt: ins_ur_istask_entry_stmt,
            ur_ingest_session_imap_account_stmt,
            ur_ingest_session_imap_acct_folder_stmt,
            ur_ingest_session_imap_acct_folder_message_stmt,
        })
    }
}

pub struct UniformResourceWriterState<'a, 'conn> {
    state_db_fs_path: &'a str,
    env_current_dir: &'a str,
    device_id: &'a str,
    ingest_session_id: &'a str,
    resources: &'a ResourcesCollection,
    ingest_stmts: &'a mut IngestContext<'conn>,
    ingest_files_behavior: Option<&'a IngestFilesBehavior>,
    ingest_fs_path_id: Option<&'a String>,
}

impl<'a, 'conn> UniformResourceWriterState<'a, 'conn> {
    fn capturable_exec_ctx(&self, entry: &mut UniformResourceWriterEntry) -> ShellStdIn {
        let path = if entry.path.is_some() {
            json!({ "path": entry.path.unwrap() })
        } else {
            json!(null)
        };
        let ctx = json!({
            "surveilr-ingest": {
                "args": { "state_db_fs_path": self.state_db_fs_path },
                "env": { "current_dir": self.env_current_dir },
                "behavior": self.ingest_files_behavior,
                "device": { "device_id": self.device_id },
                "session": {
                    "walk-session-id": self.ingest_session_id,
                    "walk-path-id": self.ingest_fs_path_id,
                    "dir-entry": path,
                },
            }
        });
        ShellStdIn::Json(ctx)
    }
}

pub struct UniformResourceWriterEntry<'a> {
    path: Option<&'a str>,
    tried_alternate_nature: Option<String>,
}

#[derive(Debug)]
pub enum UniformResourceWriterAction {
    Inserted(String, Option<String>),
    InsertedExecutableOutput(String, Option<String>, serde_json::Value),
    CapturedExecutableSqlOutput(String, serde_json::Value),
    CapturedExecutableNonZeroExit(ShellResult, serde_json::Value),
    ContentSupplierError(Box<dyn std::error::Error>),
    ContentUnavailable(),
    CapturableExecNotExecutable(),
    CapturableExecError(anyhow::Error),
    CapturableExecUrCreateError(Box<dyn std::error::Error>),
    Error(anyhow::Error),
}

impl UniformResourceWriterAction {
    fn ur_status(&self) -> Option<String> {
        match self {
            UniformResourceWriterAction::Inserted(_, ur_status) => ur_status.clone(),
            UniformResourceWriterAction::InsertedExecutableOutput(_, ur_status, _) => {
                ur_status.clone()
            }
            UniformResourceWriterAction::CapturedExecutableSqlOutput(_, _) => None,
            UniformResourceWriterAction::CapturedExecutableNonZeroExit(_, _) => {
                Some(String::from("ERROR"))
            }
            UniformResourceWriterAction::ContentSupplierError(_)
            | UniformResourceWriterAction::Error(_)
            | UniformResourceWriterAction::CapturableExecError(_)
            | UniformResourceWriterAction::CapturableExecUrCreateError(_) => {
                Some(String::from("ERROR"))
            }
            UniformResourceWriterAction::ContentUnavailable()
            | UniformResourceWriterAction::CapturableExecNotExecutable() => {
                Some(String::from("ISSUE"))
            }
        }
    }

    fn ur_diagnostics(&self) -> Option<String> {
        match self {
            UniformResourceWriterAction::Inserted(_, _) => None,
            UniformResourceWriterAction::InsertedExecutableOutput(_, _, _) => None,
            UniformResourceWriterAction::CapturedExecutableSqlOutput(_, _) => None,
            UniformResourceWriterAction::CapturedExecutableNonZeroExit(_, diags) => {
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::CapturedExecutableError(exit, stderr, diags)",
                    "message": "Non-zero exit status when executing capturable executable",
                    "diagnostics": diags // this includes exit_status and stderr already
                })).unwrap())
            }
            UniformResourceWriterAction::ContentSupplierError(err) =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::ContentSupplierError(err)",
                    "message": "Error when trying to get content from the resource",
                    "error": format!("{:?}", err)
                })).unwrap()),
            UniformResourceWriterAction::ContentUnavailable() =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::ContentUnavailable",
                    "message": "content supplier was not provided",
                    "remediation": "see CLI args/config and request content for this extension; for security reasons this service does not load any content it has not been explicitly asked to (e.g. by extension or filename pattern in behaviors)"
                })).unwrap()),
            UniformResourceWriterAction::CapturableExecNotExecutable() =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::CapturableExecNotExecutable",
                    "message": "File matched as a potential capturable executable but the file permissions do not allow execution",
                })).unwrap()),
            UniformResourceWriterAction::CapturableExecError(err) =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::CapturableExecError",
                    "message": "File matched as a potential capturable executable but could not be executed",
                    "error": err.to_string()
                })).unwrap()),
            UniformResourceWriterAction::CapturableExecUrCreateError(err) =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::CapturableExecUrCreateError",
                    "message": "File matched as a potential capturable executable and was executed but could create a new uniform resource",
                    "error": err.to_string()
                })).unwrap()),
            UniformResourceWriterAction::Error(err) =>
                Some(serde_json::to_string_pretty(&json!({
                    "message": "UniformResourceWriterAction::Error(err)",
                    "error": err.to_string()
                })).unwrap()),
        }
    }
}

#[derive(Debug)]
pub struct UniformResourceWriterResult {
    uri: String,
    action: UniformResourceWriterAction,
}

pub trait UniformResourceWriter<Resource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult;

    fn insert_text(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        resource: &ContentResource,
        _entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        let uri = resource.uri.clone();
        match resource.content_text_supplier.as_ref() {
            Some(text_supplier) => match text_supplier() {
                Ok(text) => match urw_state.ingest_stmts.ins_ur_stmt.query_row(
                    params![
                        urw_state.device_id,
                        urw_state.ingest_session_id,
                        urw_state.ingest_fs_path_id,
                        resource.uri,
                        resource.nature,
                        text.content_text(),
                        text.content_digest_hash(),
                        resource.size,
                        resource.last_modified_at.unwrap().to_string(),
                        &None::<String>, // content_fm_body_attrs
                        &None::<String>, // frontmatter
                    ],
                    |row| row.get(0),
                ) {
                    Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                        uri,
                        action: UniformResourceWriterAction::Inserted(new_or_existing_ur_id, None),
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
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        resource: &ContentResource,
        bc: Box<dyn BinaryContent>,
        _entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        let uri = resource.uri.clone();
        match urw_state.ingest_stmts.ins_ur_stmt.query_row(
            params![
                urw_state.device_id,
                urw_state.ingest_session_id,
                urw_state.ingest_fs_path_id,
                resource.uri,
                resource.nature,
                bc.content_binary(),
                bc.content_digest_hash(),
                resource.size,
                resource.last_modified_at.unwrap().to_string(),
                &None::<String>, // content_fm_body_attrs
                &None::<String>, // frontmatter
            ],
            |row| row.get(0),
        ) {
            Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Inserted(new_or_existing_ur_id, None),
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
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        let uri = self.uri.clone();
        match urw_state.ingest_stmts.ins_ur_stmt.query_row(
            params![
                urw_state.device_id,
                urw_state.ingest_session_id,
                urw_state.ingest_fs_path_id,
                self.uri,
                self.nature,
                &None::<String>,   // not storing content
                String::from("-"), // no hash being computed
                self.size,
                self.last_modified_at.unwrap().to_string(),
                &None::<String>, // content_fm_body_attrs
                &None::<String>, // frontmatter
            ],
            |row| row.get(0),
        ) {
            Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Inserted(
                    new_or_existing_ur_id,
                    Some(format!(
                        "UKNOWN_NATURE({})",
                        if let Some(alternate) = entry.tried_alternate_nature.clone() {
                            alternate
                        } else {
                            self.nature.clone().unwrap_or("?".to_string())
                        }
                    )),
                ),
            },
            Err(err) => UniformResourceWriterResult {
                uri,
                action: UniformResourceWriterAction::Error(err.into()),
            },
        }
    }
}

impl UniformResourceWriter<ContentResource> for CapturableExecResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        // if resources collection instance wants to, store the executable as a uniform_resource itself so we have history;
        self.insert_text(urw_state, &self.resource, entry);

        // now try to execute the capturable executable and store its output
        match &self.executable {
            CapturableExecutable::UriShellExecutive(
                executive,
                interpretable_code,
                nature,
                is_batched_sql,
            ) => {
                let stdin = urw_state.capturable_exec_ctx(entry);
                match executive.execute(stdin.clone()) {
                    Ok(shell_result) => {
                        let captured_executable_diags = json!({
                            "args": [],
                            "interpretable-code": interpretable_code,
                            "stdin": stdin.json(),
                            "exit-status": format!("{:?}", shell_result.status),
                            "stderr": shell_result.stderr,
                        });

                        if shell_result.success() {
                            if *is_batched_sql {
                                // the text is considered SQL and should be executed by the
                                // caller so we do not store anything in uniform_resource here.
                                return UniformResourceWriterResult {
                                    uri: self.resource.uri.clone(),
                                    action:
                                        UniformResourceWriterAction::CapturedExecutableSqlOutput(
                                            shell_result.stdout,
                                            captured_executable_diags,
                                        ),
                                };
                            }

                            let hash = shell_result.stdout_hash();
                            let output_res = ContentResource {
                                flags: self.resource.flags,
                                uri: self.resource.uri.clone(),
                                nature: Some(nature.clone()),
                                size: Some(shell_result.stdout.len().try_into().unwrap()),
                                created_at: Some(chrono::Utc::now()),
                                last_modified_at: Some(chrono::Utc::now()),
                                content_binary_supplier: None,
                                content_text_supplier: Some(Box::new(
                                    move || -> Result<Box<dyn TextContent>, Box<dyn std::error::Error>> {
                                        // TODO: do we really need to make clone these, can't we just
                                        // pass in self.executable.capturable_exec_text_supplier!?!?
                                        Ok(Box::new(ResourceTextContent { text: shell_result.stdout.clone(), hash: hash.clone() })
                                            as Box<dyn TextContent>)
                                    },
                                )),
                            };

                            match urw_state.resources.uniform_resource(output_res) {
                                Ok(output_ur) => {
                                    let ur = *(output_ur);
                                    let inserted_output =
                                        insert_uniform_resource(&ur, urw_state, entry);
                                    match inserted_output.action {
                                        UniformResourceWriterAction::Inserted(ur_id, ur_status) => {
                                            UniformResourceWriterResult {
                                                uri: inserted_output.uri,
                                                action: UniformResourceWriterAction::InsertedExecutableOutput(ur_id, ur_status,
                                                    captured_executable_diags),
                                            }
                                        },
                                        _ => inserted_output
                                    }
                                }
                                Err(err) => UniformResourceWriterResult {
                                    uri: self.resource.uri.clone(),
                                    action:
                                        UniformResourceWriterAction::CapturableExecUrCreateError(
                                            err,
                                        ),
                                },
                            }
                        } else {
                            UniformResourceWriterResult {
                                uri: self.resource.uri.clone(),
                                action: UniformResourceWriterAction::CapturedExecutableNonZeroExit(
                                    shell_result,
                                    captured_executable_diags,
                                ),
                            }
                        }
                    }
                    Err(err) => UniformResourceWriterResult {
                        uri: self.resource.uri.clone(),
                        action: UniformResourceWriterAction::CapturableExecError(err),
                    },
                }
            }
            CapturableExecutable::RequestedButNotExecutable(_src) => UniformResourceWriterResult {
                uri: self.resource.uri.clone(),
                action: UniformResourceWriterAction::CapturableExecNotExecutable(),
            },
        }
    }
}

impl UniformResourceWriter<ContentResource> for HtmlResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for ImageResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        let uri = self.resource.uri.clone();
        match self.resource.content_binary_supplier.as_ref() {
            Some(image_supplier) => match image_supplier() {
                Ok(image_src) => self.insert_binary(urw_state, &self.resource, image_src, entry),
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
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for JsonableTextResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for MarkdownResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        _entry: &mut UniformResourceWriterEntry,
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
                        // this needs to be JSON parse'd first so that stringify later will work properly
                        let attrs_json: serde_json::Value =
                            serde_json::from_str(&fm_json.clone().unwrap()).unwrap();
                        let fm_attrs_value = serde_json::json!({
                            "frontMatter": fm_raw.unwrap(),
                            "body": fm_body,
                            "attrs": attrs_json
                        });
                        fm_attrs = Some(serde_json::to_string_pretty(&fm_attrs_value).unwrap());
                    }
                    let uri = self.resource.uri.to_string();
                    match urw_state.ingest_stmts.ins_ur_stmt.query_row(
                        params![
                            urw_state.device_id,
                            urw_state.ingest_session_id,
                            urw_state.ingest_fs_path_id,
                            self.resource.uri,
                            self.resource.nature,
                            markdown_src.content_text(),
                            markdown_src.content_digest_hash(),
                            self.resource.size,
                            self.resource.last_modified_at.unwrap().to_string(),
                            fm_attrs,
                            fm_json,
                        ],
                        |row| row.get(0),
                    ) {
                        Ok(new_or_existing_ur_id) => UniformResourceWriterResult {
                            uri,
                            action: UniformResourceWriterAction::Inserted(
                                new_or_existing_ur_id,
                                None,
                            ),
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

impl UniformResourceWriter<ContentResource> for PlainTextResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for SourceCodeResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for XmlResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        let uri = &self.resource.uri;
        let ur_res = self.insert_text(urw_state, &self.resource, entry);

        if let UniformResourceWriterAction::Inserted(ur_id, _) = &ur_res.action {
            let (json, hash) = match self.transform_to_json() {
                Ok(s) => s,
                Err(err) => {
                    return UniformResourceWriterResult {
                        uri: uri.to_string(),
                        action: UniformResourceWriterAction::Error(err),
                    };
                }
            };

            match urw_state.ingest_stmts.ins_ur_transform_stmt.query_row(
                params![
                    ur_id,
                    self.resource.uri.to_string(),
                    "json".to_string(),
                    hash,
                    json,
                    json.as_bytes().len()
                ],
                |row| row.get::<_, String>(0),
            ) {
                Ok(id) => UniformResourceWriterResult {
                    uri: uri.to_string(),
                    action: UniformResourceWriterAction::Inserted(id, None),
                },
                Err(err) => UniformResourceWriterResult {
                    uri: uri.to_string(),
                    action: UniformResourceWriterAction::Error(err.into()),
                },
            }
        } else {
            ur_res
        }
    }
}

impl UniformResourceWriter<ContentResource> for ImapResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

fn insert_uniform_resource(
    resource: &UniformResource<ContentResource>,
    urw_state: &mut UniformResourceWriterState<'_, '_>,
    entry: &mut UniformResourceWriterEntry,
) -> UniformResourceWriterResult {
    match resource {
        UniformResource::CapturableExec(capturable) => capturable.insert(urw_state, entry),
        UniformResource::Html(html) => html.insert(urw_state, entry),
        UniformResource::Json(json) => json.insert(urw_state, entry),
        UniformResource::JsonableText(jtr) => jtr.insert(urw_state, entry),
        UniformResource::Image(img) => img.insert(urw_state, entry),
        UniformResource::Markdown(md) => md.insert(urw_state, entry),
        UniformResource::PlainText(txt) => txt.insert(urw_state, entry),
        UniformResource::SourceCode(sc) => sc.insert(urw_state, entry),
        UniformResource::Xml(xml) => xml.insert(urw_state, entry),
        UniformResource::ImapResource(imap) => imap.insert(urw_state, entry),
        UniformResource::Unknown(unknown, tried_alternate_nature) => {
            if let Some(tried_alternate_nature) = tried_alternate_nature {
                entry.tried_alternate_nature = Some(tried_alternate_nature.clone());
            }
            unknown.insert(urw_state, entry)
        }
    }
}

// impl UniformResource<ContentResource> {
//     fn insert(
//         &self,
//         urw_state: &mut UniformResourceWriterState<'_, '_>,
//         entry: &mut UniformResourceWriterEntry,
//     ) -> UniformResourceWriterResult {
//         match self {
//             UniformResource::CapturableExec(capturable) => capturable.insert(urw_state, entry),
//             UniformResource::Html(html) => html.insert(urw_state, entry),
//             UniformResource::Json(json) => json.insert(urw_state, entry),
//             UniformResource::JsonableText(jtr) => jtr.insert(urw_state, entry),
//             UniformResource::Image(img) => img.insert(urw_state, entry),
//             UniformResource::Markdown(md) => md.insert(urw_state, entry),
//             UniformResource::PlainText(txt) => txt.insert(urw_state, entry),
//             UniformResource::SourceCode(sc) => sc.insert(urw_state, entry),
//             UniformResource::Xml(xml) => xml.insert(urw_state, entry),
//             UniformResource::Unknown(unknown, tried_alternate_nature) => {
//                 if let Some(tried_alternate_nature) = tried_alternate_nature {
//                     entry.tried_alternate_nature = Some(tried_alternate_nature.clone());
//                 }
//                 unknown.insert(urw_state, entry)
//             }
//         }
//     }
// }

#[derive(Serialize, Deserialize)]
pub struct IngestFilesBehavior {
    pub classifier: EncounterableResourcePathClassifier,
    pub root_fs_paths: Vec<String>,
}

impl IngestFilesBehavior {
    // #[autometrics]
    pub fn new(
        device_id: &String,
        ingest_args: &IngestFilesArgs,
        conn: &Connection,
    ) -> anyhow::Result<(Self, Option<String>)> {
        if let Some(behavior_name) = &ingest_args.behavior {
            let (behavior_id, behavior_json): (String, String) = conn
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
                        "[IngestFilesBehavior.new] unable to read behavior '{}' from {} behavior table",
                        behavior_name, ingest_args.state_db_fs_path
                    )
                })?;
            let behavior = IngestFilesBehavior::from_json(&behavior_json).with_context(|| {
                format!(
                    "[IngestFilesBehavior.new] unable to deserialize behavior {} in {}",
                    behavior_json, ingest_args.state_db_fs_path
                )
            })?;
            Ok((behavior, Some(behavior_id)))
        } else {
            Ok((
                IngestFilesBehavior::from_ingest_args(ingest_args, conn)?,
                None,
            ))
        }
    }

    // #[autometrics]
    pub fn from_ingest_args(args: &IngestFilesArgs, conn: &Connection) -> anyhow::Result<Self> {
        // the names in `args` are convenient for CLI usage but the struct
        // field names in IngestBehavior should be longer and more descriptive
        // since IngestBehavior is stored as activity in the database.
        Ok(IngestFilesBehavior {
            classifier: EncounterableResourcePathClassifier::default_from_conn(conn)?,
            root_fs_paths: args.root_fs_path.clone(),
        })
    }

    #[autometrics]
    pub fn from_json(json_text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_text)
    }

    #[autometrics]
    pub fn persistable_json_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    #[autometrics]
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
            .with_context(|| {
                format!(
                    "[IngestFilesBehavior.save] unable to save behavior '{}'",
                    behavior_name
                )
            })?;
        Ok(behavior_id)
    }
}

// #[autometrics]

#[derive(Serialize, Deserialize)]
pub struct IngestTasksBehavior {
    pub lines: Vec<String>,         // what was given
    pub encounterable: Vec<String>, // after filtering for comments, blanks, etc.
}

impl IngestTasksBehavior {
    #[autometrics]
    pub fn from_stdin() -> Self {
        let lines: Vec<_> = std::io::stdin()
            .lines()
            .map(Result::ok)
            .map(|t| t.unwrap())
            .collect();
        IngestTasksBehavior {
            lines: lines.clone(),
            encounterable: lines,
        }
    }

    #[autometrics]
    pub fn persistable_json_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
