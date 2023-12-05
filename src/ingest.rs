use std::collections::HashMap;

use anyhow::{Context, Result};
use indoc::indoc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::persist::*;
use crate::resource::*;
use crate::shell::*;

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

#[allow(dead_code)]
#[derive(Debug)]
pub struct IngestContext<'conn> {
    ins_ur_isfsp_stmt: rusqlite::Statement<'conn>,
    ins_ur_stmt: rusqlite::Statement<'conn>,
    ins_ur_transform_stmt: rusqlite::Statement<'conn>,
    ins_ur_isfsp_entry_stmt: rusqlite::Statement<'conn>,
    ins_ur_is_task_stmt: rusqlite::Statement<'conn>,
}

impl<'conn> IngestContext<'conn> {
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
        Ok(IngestContext {
            ins_ur_isfsp_stmt,
            ins_ur_stmt,
            ins_ur_transform_stmt,
            ins_ur_isfsp_entry_stmt,
            ins_ur_is_task_stmt: ins_ur_istask_entry_stmt,
        })
    }
}

pub struct UniformResourceWriterState<'a, 'conn> {
    state_db_fs_path: &'a String,
    env_current_dir: &'a String,
    device_id: &'a String,
    ingest_session_id: &'a String,
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
                                    let inserted_output = ur.insert(urw_state, entry);
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
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        match self {
            UniformResource::CapturableExec(capturable) => capturable.insert(urw_state, entry),
            UniformResource::Html(html) => html.insert(urw_state, entry),
            UniformResource::Json(json) => json.insert(urw_state, entry),
            UniformResource::JsonableText(jtr) => jtr.insert(urw_state, entry),
            UniformResource::Image(img) => img.insert(urw_state, entry),
            UniformResource::Markdown(md) => md.insert(urw_state, entry),
            UniformResource::PlainText(txt) => txt.insert(urw_state, entry),
            UniformResource::SourceCode(sc) => sc.insert(urw_state, entry),
            UniformResource::Xml(xml) => xml.insert(urw_state, entry),
            UniformResource::Unknown(unknown, tried_alternate_nature) => {
                if let Some(tried_alternate_nature) = tried_alternate_nature {
                    entry.tried_alternate_nature = Some(tried_alternate_nature.clone());
                }
                unknown.insert(urw_state, entry)
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct IngestFilesBehavior {
    pub classifier: EncounterableResourcePathClassifier,
    pub root_fs_paths: Vec<String>,
}

impl IngestFilesBehavior {
    pub fn new(
        device_id: &String,
        ingest_args: &crate::cmd::IngestFilesArgs,
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

    pub fn from_ingest_args(
        args: &crate::cmd::IngestFilesArgs,
        conn: &Connection,
    ) -> anyhow::Result<Self> {
        // the names in `args` are convenient for CLI usage but the struct
        // field names in IngestBehavior should be longer and more descriptive
        // since IngestBehavior is stored as activity in the database.
        Ok(IngestFilesBehavior {
            classifier: EncounterableResourcePathClassifier::default_from_conn(conn)?,
            root_fs_paths: args.root_fs_path.clone(),
        })
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
            .with_context(|| {
                format!(
                    "[IngestFilesBehavior.save] unable to save behavior '{}'",
                    behavior_name
                )
            })?;
        Ok(behavior_id)
    }
}

pub fn ingest_files(
    cli: &crate::cmd::Cli,
    ingest_args: &crate::cmd::IngestFilesArgs,
) -> Result<String> {
    let mut dbc = DbConn::new(&ingest_args.state_db_fs_path, cli.debug).with_context(|| {
        format!(
            "[ingest_files] SQLite transaction in {}",
            ingest_args.state_db_fs_path
        )
    })?;
    let db_fs_path = dbc.db_fs_path.clone();

    // putting everything inside a transaction improves performance significantly
    let tx = dbc.init(Some(&ingest_args.state_db_init_sql))?;
    let (device_id, _device_name) = upserted_device(&tx, &crate::DEVICE).with_context(|| {
        format!(
            "[ingest_files] upserted_device {} in {}",
            crate::DEVICE.name,
            db_fs_path
        )
    })?;

    // the ulid() function we're using below is not built into SQLite, we define
    // it in persist::prepare_conn so it's initialized as part of `dbc`.

    let (mut behavior, mut behavior_id) = IngestFilesBehavior::new(&device_id, ingest_args, &tx)
        .with_context(|| format!("[ingest_files] behavior issue {}", db_fs_path))?;

    if !ingest_args.include_state_db_in_ingestion {
        let canonical_db_fs_path = std::fs::canonicalize(std::path::Path::new(&db_fs_path))
            .with_context(|| format!("[ingest_files] unable to canonicalize in {}", db_fs_path))?;
        let canonical_db_fs_path = canonical_db_fs_path.to_string_lossy().to_string();
        let mut wal_path = std::path::PathBuf::from(&canonical_db_fs_path);
        let mut db_journal_path = std::path::PathBuf::from(&canonical_db_fs_path);
        wal_path.set_extension("wal");
        db_journal_path.set_extension("db-journal");
        behavior
            .classifier
            .add_ignore_exact(canonical_db_fs_path.as_str());
        behavior
            .classifier
            .add_ignore_exact(wal_path.to_string_lossy().to_string().as_str());
        behavior
            .classifier
            .add_ignore_exact(db_journal_path.to_string_lossy().to_string().as_str());
    }

    if let Some(save_behavior_name) = &ingest_args.save_behavior {
        let saved_bid = behavior
            .save(&tx, &device_id, save_behavior_name)
            .with_context(|| {
                format!(
                    "[ingest_files] saving {} in {}",
                    save_behavior_name, db_fs_path
                )
            })?;
        if cli.debug > 0 {
            println!("Saved behavior: {} ({})", save_behavior_name, saved_bid);
        }
        behavior_id = Some(saved_bid);
    }
    if cli.debug > 0 {
        println!(
            "Behavior: {}",
            behavior_id.clone().unwrap_or(String::from("custom"))
        );
    }

    let ingest_session_id: String = tx
        .query_row(
            INS_UR_INGEST_SESSION_SQL,
            params![
                device_id,
                behavior_id,
                match behavior.persistable_json_text() {
                    Ok(json_text) => json_text,
                    Err(_err) =>
                        String::from("JSON serialization error, TODO: convert err to string"),
                }
            ],
            |row| row.get(0),
        )
        .with_context(|| {
            format!(
                "[ingest_files] inserting UR walk session using {} in {}",
                INS_UR_INGEST_SESSION_SQL, db_fs_path
            )
        })?;

    if cli.debug > 0 {
        println!("Walk Session: {ingest_session_id}");
    }
    {
        let env_current_dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut ingest_stmts = IngestContext::from_conn(&tx, &ingest_args.state_db_fs_path)
            .with_context(|| format!("[ingest_files] ingest_stmts in {}", db_fs_path))?;

        for root_path in &behavior.root_fs_paths {
            let canonical_path_buf = std::fs::canonicalize(std::path::Path::new(&root_path))
                .with_context(|| {
                    format!(
                        "[ingest_files] unable to canonicalize {} in {}",
                        root_path, db_fs_path
                    )
                })?;
            let canonical_path = canonical_path_buf.into_os_string().into_string().unwrap();

            let ins_ur_wsp_params = params![ingest_session_id, canonical_path];
            let ingest_fs_path_id: String = ingest_stmts
                .ins_ur_isfsp_stmt
                .query_row(ins_ur_wsp_params, |row| row.get(0))
                .with_context(|| {
                    format!(
                        "[ingest_files] ins_ur_wsp_stmt {} with {} in {}",
                        INS_UR_ISFSP_SQL, "TODO: ins_ur_wsp_params.join()", db_fs_path
                    )
                })?;

            if cli.debug > 0 {
                println!("  Walk Session Path: {root_path} ({ingest_fs_path_id})");
            }

            let rp: Vec<String> = vec![canonical_path.clone()];
            let resources = ResourcesCollection::from_smart_ignore(
                &rp,
                &behavior.classifier,
                &None::<HashMap<_, _>>,
                false,
            );

            let mut urw_state = UniformResourceWriterState {
                state_db_fs_path: &db_fs_path,
                ingest_files_behavior: Some(&behavior),
                env_current_dir: &env_current_dir,
                device_id: &device_id,
                ingest_session_id: &ingest_session_id,
                ingest_fs_path_id: Some(&ingest_fs_path_id),
                resources: &resources,
                ingest_stmts: &mut ingest_stmts,
            };

            for resource_result in resources.uniform_resources() {
                match resource_result {
                    Ok(resource) => {
                        let mut urw_entry = UniformResourceWriterEntry {
                            path: Some(resource.uri()),
                            tried_alternate_nature: None,
                        };
                        let inserted = resource.insert(&mut urw_state, &mut urw_entry);
                        let mut ur_status = inserted.action.ur_status();
                        let mut ur_diagnostics = inserted.action.ur_diagnostics();
                        let mut captured_exec_diags: Option<String> = None;

                        let uniform_resource_id = match &inserted.action {
                            UniformResourceWriterAction::Inserted(
                                ref uniform_resource_id,
                                None,
                            ) => Some(uniform_resource_id),
                            UniformResourceWriterAction::InsertedExecutableOutput(
                                ref uniform_resource_id,
                                None,
                                diags,
                            ) => {
                                captured_exec_diags =
                                    Some(serde_json::to_string_pretty(&diags).unwrap());
                                Some(uniform_resource_id)
                            }
                            UniformResourceWriterAction::CapturedExecutableSqlOutput(
                                ref sql_script,
                                diags,
                            ) => {
                                captured_exec_diags =
                                    Some(serde_json::to_string_pretty(&diags).unwrap());
                                match tx.execute_batch(sql_script) {
                                    Ok(_) => {
                                        ur_status = Some(String::from("EXECUTED_CAPTURED_SQL"));
                                        ur_diagnostics = Some(serde_json::to_string_pretty(&json!({
                                            "instance": "UniformResourceWriterAction::CapturedExecutableSqlOutput(err)",
                                            "SQL": sql_script
                                        })).unwrap());
                                        None
                                    }
                                    Err(err) => {
                                        ur_status = Some(String::from("ERROR"));
                                        ur_diagnostics = Some(serde_json::to_string_pretty(&json!({
                                            "instance": "UniformResourceWriterAction::CapturedExecutableSqlOutput(err)",
                                            "message": "Error executing batched SQL",
                                            "error": err.to_string(),
                                            "SQL": sql_script
                                        })).unwrap());
                                        None
                                    }
                                }
                            }
                            _ => None,
                        };

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
                                match urw_state.ingest_stmts.ins_ur_isfsp_entry_stmt.execute(
                                    params![
                                        ingest_session_id,
                                        ingest_fs_path_id,
                                        uniform_resource_id,
                                        file_path_abs.into_os_string().into_string().unwrap(),
                                        file_path_rel_parent
                                            .into_os_string()
                                            .into_string()
                                            .unwrap(),
                                        file_path_rel.into_os_string().into_string().unwrap(),
                                        file_basename,
                                        if let Some(file_extn) = file_extn {
                                            file_extn
                                        } else {
                                            String::from("")
                                        },
                                        ur_status,
                                        ur_diagnostics,
                                        captured_exec_diags
                                    ],
                                ) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        eprintln!( "[ingest_files] unable to insert UR walk session path file system entry for {} in {}: {} ({})",
                                        &inserted.uri, db_fs_path, err, INS_UR_ISFSP_ENTRY_SQL
                                        )
                                    }
                                }
                            }
                            None => {
                                eprintln!(
                                    "[ingest_files] error extracting path info for {} in {}",
                                    canonical_path, db_fs_path
                                )
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("[ingest_files] Error processing a resource: {}", e);
                    }
                }
            }
        }
    }
    match tx.execute(INS_UR_INGEST_SESSION_FINISH_SQL, params![ingest_session_id]) {
        Ok(_) => {}
        Err(err) => {
            eprintln!(
                "[ingest_files] unable to execute SQL {} in {}: {}",
                INS_UR_INGEST_SESSION_FINISH_SQL, db_fs_path, err
            )
        }
    }
    // putting everything inside a transaction improves performance significantly
    tx.commit().with_context(|| {
        format!(
            "[ingest_files] unable to perform final commit in {}",
            db_fs_path
        )
    })?;

    Ok(ingest_session_id)
}

#[derive(Serialize, Deserialize)]
pub struct IngestTasksBehavior {
    pub lines: Vec<String>,         // what was given
    pub encounterable: Vec<String>, // after filtering for comments, blanks, etc.
}

impl IngestTasksBehavior {
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

    pub fn persistable_json_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

pub fn ingest_tasks(
    cli: &crate::cmd::Cli,
    ingest_args: &crate::cmd::IngestTasksArgs,
) -> Result<String> {
    let mut dbc = DbConn::new(&ingest_args.state_db_fs_path, cli.debug).with_context(|| {
        format!(
            "[ingest_tasks] SQLite transaction in {}",
            ingest_args.state_db_fs_path
        )
    })?;
    let db_fs_path = dbc.db_fs_path.clone();

    // putting everything inside a transaction improves performance significantly
    let tx = dbc.init(Some(&ingest_args.state_db_init_sql))?;
    let (device_id, _device_name) = upserted_device(&tx, &crate::DEVICE).with_context(|| {
        format!(
            "[ingest_tasks] upserted_device {} in {}",
            crate::DEVICE.name,
            db_fs_path
        )
    })?;

    let mut behavior = IngestTasksBehavior::from_stdin();
    let classifier = EncounterableResourcePathClassifier::default_from_conn(&tx)?;
    let (encounterable, resources) =
        ResourcesCollection::from_tasks_lines(&behavior.lines, &classifier, &None::<HashMap<_, _>>);
    behavior.encounterable = encounterable;

    let ingest_session_id: String = tx
        .query_row(
            INS_UR_INGEST_SESSION_SQL,
            params![
                device_id,
                None::<String>,
                match behavior.persistable_json_text() {
                    Ok(json_text) => json_text,
                    Err(_err) =>
                        String::from("JSON serialization error, TODO: convert err to string"),
                }
            ],
            |row| row.get(0),
        )
        .with_context(|| {
            format!(
                "[ingest_tasks] inserting UR walk session using {} in {}",
                INS_UR_INGEST_SESSION_SQL, db_fs_path
            )
        })?;
    if cli.debug > 0 {
        println!("Walk Session: {ingest_session_id}");
    }

    {
        let env_current_dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut ingest_stmts = IngestContext::from_conn(&tx, &ingest_args.state_db_fs_path)
            .with_context(|| format!("[ingest_tasks] ingest_stmts in {}", db_fs_path))?;

        let mut urw_state = UniformResourceWriterState {
            state_db_fs_path: &db_fs_path,
            ingest_files_behavior: None,
            env_current_dir: &env_current_dir,
            device_id: &device_id,
            ingest_session_id: &ingest_session_id,
            ingest_fs_path_id: None,
            resources: &resources,
            ingest_stmts: &mut ingest_stmts,
        };

        for resource_result in resources.uniform_resources() {
            match resource_result {
                Ok(resource) => {
                    let mut urw_entry = UniformResourceWriterEntry {
                        path: Some(resource.uri()),
                        tried_alternate_nature: None,
                    };
                    if cli.debug > 0 {
                        println!("{:?}", urw_entry.path);
                    }

                    let inserted = resource.insert(&mut urw_state, &mut urw_entry);
                    let mut ur_status = inserted.action.ur_status();
                    let mut ur_diagnostics = inserted.action.ur_diagnostics();
                    let captured_executable: Option<String>;

                    let uniform_resource_id = match &inserted.action {
                        UniformResourceWriterAction::InsertedExecutableOutput(
                            ref uniform_resource_id,
                            _,
                            diags,
                        ) => {
                            captured_executable =
                                Some(serde_json::to_string_pretty(&diags).unwrap());
                            Some(uniform_resource_id)
                        }
                        UniformResourceWriterAction::CapturedExecutableSqlOutput(
                            ref sql_script,
                            diags,
                        ) => {
                            captured_executable =
                                Some(serde_json::to_string_pretty(&diags).unwrap());
                            match tx.execute_batch(sql_script) {
                                Ok(_) => {
                                    ur_status = Some(String::from("EXECUTED_CAPTURED_SQL"));
                                    ur_diagnostics = Some(serde_json::to_string_pretty(&json!({
                                            "instance": "UniformResourceWriterAction::CapturedExecutableSqlOutput(err)",
                                            "SQL": sql_script
                                        })).unwrap());
                                    None
                                }
                                Err(err) => {
                                    ur_status = Some(String::from("ERROR"));
                                    ur_diagnostics = Some(serde_json::to_string_pretty(&json!({
                                            "instance": "UniformResourceWriterAction::CapturedExecutableSqlOutput(err)",
                                            "message": "Error executing batched SQL",
                                            "error": err.to_string(),
                                            "SQL": sql_script
                                        })).unwrap());
                                    None
                                }
                            }
                        }
                        _ => {
                            ur_status = Some(String::from("ERROR"));
                            captured_executable = Some(
                                r#"{ "error": "captured_executable should never be set in this condition" }"#
                                    .to_owned(),
                            );
                            None
                        }
                    };

                    match urw_state.ingest_stmts.ins_ur_is_task_stmt.execute(params![
                        ingest_session_id,
                        uniform_resource_id,
                        captured_executable,
                        ur_status,
                        ur_diagnostics,
                    ]) {
                        Ok(_) => {}
                        Err(err) => {
                            eprintln!( "[ingest_tasks] unable to insert UR task entry for {} in {}: {} ({})",
                            &inserted.uri, db_fs_path, err, INS_UR_IS_TASK_SQL
                            )
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error processing a ingest_tasks resource: {}", e);
                }
            }
        }
    }

    match tx.execute(INS_UR_INGEST_SESSION_FINISH_SQL, params![ingest_session_id]) {
        Ok(_) => {}
        Err(err) => {
            eprintln!(
                "[ingest_tasks] unable to execute SQL {} in {}: {}",
                INS_UR_INGEST_SESSION_FINISH_SQL, db_fs_path, err
            )
        }
    }

    // putting everything inside a transaction improves performance significantly
    tx.commit().with_context(|| {
        format!(
            "[ingest_tasks] unable to perform final commit in {}",
            db_fs_path
        )
    })?;

    Ok(ingest_session_id)
}
