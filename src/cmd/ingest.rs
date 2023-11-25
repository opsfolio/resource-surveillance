use std::collections::HashMap;

use anyhow::{Context, Result};
use indoc::indoc;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_regex;
use walkdir::DirEntry;

use crate::capturable::*;
use crate::fscontent::*;
use crate::fsresource::*;
use crate::persist::*;
use crate::resource::*;

pub struct UniformResourceWriterState<'a, 'conn> {
    ingest_args: &'a super::IngestArgs,
    env_current_dir: &'a String,
    ingest_behavior: &'a IngestBehavior,
    device_id: &'a String,
    ingest_session_id: &'a String,
    ingest_fs_path_id: &'a String,
    fsr_walker: &'a FileSysResourcesWalker,
    ins_ur_stmt: &'a mut rusqlite::Statement<'conn>,
    _ins_ur_transform_stmt: &'a mut rusqlite::Statement<'conn>,
}

impl<'a, 'conn> UniformResourceWriterState<'a, 'conn> {
    fn capturable_exec_ctx(&self, entry: &mut UniformResourceWriterEntry) -> Option<String> {
        let ctx = json!({
            "surveilr-ingest": {
                "args": { "state_db_fs_path": self.ingest_args.state_db_fs_path },
                "env": { "current_dir": self.env_current_dir },
                "behavior": self.ingest_behavior,
                "device": { "device_id": self.device_id },
                "session": {
                    "walk-session-id": self.ingest_session_id,
                    "walk-path-id": self.ingest_fs_path_id,
                    "entry": { "path": entry.dir_entry.path().to_str().unwrap() },
                },
            }
        });
        Some(serde_json::to_string_pretty(&ctx).unwrap())
    }
}

pub struct UniformResourceWriterEntry<'a> {
    dir_entry: &'a DirEntry,
    tried_alternate_nature: Option<String>,
}

#[derive(Debug)]
pub enum UniformResourceWriterAction {
    Inserted(String, Option<String>),
    InsertedExecutableOutput(String, Option<String>, serde_json::Value),
    CapturedExecutableSqlOutput(String, serde_json::Value),
    CapturedExecutableNonZeroExit(subprocess::ExitStatus, Option<String>, serde_json::Value),
    ContentSupplierError(Box<dyn std::error::Error>),
    ContentUnavailable(),
    CapturableExecNotExecutable(),
    CapturableExecNoCaptureSupplier(),
    CapturableExecNoNature(regex::Regex),
    CapturableExecError(Box<dyn std::error::Error>),
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
            UniformResourceWriterAction::CapturedExecutableNonZeroExit(_, _, _) => {
                Some(String::from("ERROR"))
            }
            UniformResourceWriterAction::ContentSupplierError(_)
            | UniformResourceWriterAction::Error(_)
            | UniformResourceWriterAction::CapturableExecError(_)
            | UniformResourceWriterAction::CapturableExecUrCreateError(_) => {
                Some(String::from("ERROR"))
            }
            UniformResourceWriterAction::CapturableExecNoNature(_) => Some(String::from("ISSUE")),
            UniformResourceWriterAction::ContentUnavailable()
            | UniformResourceWriterAction::CapturableExecNotExecutable()
            | UniformResourceWriterAction::CapturableExecNoCaptureSupplier() => {
                Some(String::from("ISSUE"))
            }
        }
    }

    fn ur_diagnostics(&self) -> Option<String> {
        match self {
            UniformResourceWriterAction::Inserted(_, _) => None,
            UniformResourceWriterAction::InsertedExecutableOutput(_, _, _) => None,
            UniformResourceWriterAction::CapturedExecutableSqlOutput(_, _) => None,
            UniformResourceWriterAction::CapturedExecutableNonZeroExit(_, _, diags) => {
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
            UniformResourceWriterAction::CapturableExecNoNature(re)  =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::CapturableExecNoNature",
                    "message": "File matched as a potential capturable executable but no 'nature' capture was found in RegEx",
                    "regex": re.to_string()
                })).unwrap()),
            UniformResourceWriterAction::CapturableExecNoCaptureSupplier() =>
                Some(serde_json::to_string_pretty(&json!({
                    "instance": "UniformResourceWriterAction::CapturableExecNoCaptureSupplier",
                    "message": "File matched as a potential capturable executable but no capture supplier was provided",
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
                Ok(text) => match urw_state.ins_ur_stmt.query_row(
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
        match urw_state.ins_ur_stmt.query_row(
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
        match urw_state.ins_ur_stmt.query_row(
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
        // if FsWalker wants to, store the executable as a uniform_resource itself so we have history;
        self.insert_text(urw_state, &self.executable, entry);

        // now try to execute the capturable executable and store its output
        match &self.executable.capturable_executable {
            Some(capturable_executable) => match capturable_executable {
                CapturableExecutable::Text(nature, is_batched_sql) => {
                    match self.executable.capturable_exec_text_supplier.as_ref() {
                        Some(capturable_supplier) => {
                            let stdin = urw_state.capturable_exec_ctx(entry);
                            match capturable_supplier(stdin.clone()) {
                                Ok((capture_src, exit_status, stderr)) => {
                                    let stdin_json: serde_json::Value =
                                        serde_json::from_str(stdin.unwrap().as_str()).unwrap();
                                    let captured_executable_diags = json!({
                                        "args": [],
                                        "stdin": stdin_json,
                                        "exit-status": format!("{:?}", exit_status),
                                        "stderr": stderr,
                                    });

                                    if matches!(exit_status, subprocess::ExitStatus::Exited(0)) {
                                        let captured_text =
                                            String::from(capture_src.content_text());
                                        if *is_batched_sql {
                                            // the text is considered SQL and should be executed by the
                                            // caller so we do not store anything in uniform_resource here.
                                            return UniformResourceWriterResult {
                                                uri: self.executable.uri.clone(),
                                                action: UniformResourceWriterAction::CapturedExecutableSqlOutput(captured_text, captured_executable_diags)
                                            };
                                        }

                                        let hash = String::from(capture_src.content_digest_hash());
                                        let output_res = ContentResource {
                                            uri: self.executable.uri.clone(),
                                            nature: Some(nature.clone()),
                                            size: Some(captured_text.len().try_into().unwrap()),
                                            created_at: Some(chrono::Utc::now()),
                                            last_modified_at: Some(chrono::Utc::now()),
                                            content_binary_supplier: None,
                                            content_text_supplier: Some(Box::new(
                                                move || -> Result<Box<dyn TextContent>, Box<dyn std::error::Error>> {
                                                    // TODO: do we really need to make clone these, can't we just
                                                    // pass in self.executable.capturable_exec_text_supplier!?!?
                                                    Ok(Box::new(FileTextContent { text: captured_text.clone(), hash: hash.clone() })
                                                        as Box<dyn TextContent>)
                                                },
                                            )),
                                            capturable_executable: None,
                                            capturable_exec_binary_supplier: None,
                                            capturable_exec_text_supplier: None,
                                        };

                                        match urw_state.fsr_walker.resource_supplier.uniform_resource(output_res) {
                                            Ok(output_ur) => {
                                                let ur = *(output_ur);
                                                let inserted_output = ur.insert( urw_state, entry);
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
                                                uri: self.executable.uri.clone(),
                                                action:
                                                    UniformResourceWriterAction::CapturableExecUrCreateError(
                                                        err,
                                                    ),
                                            },
                                        }
                                    } else {
                                        UniformResourceWriterResult {
                                            uri: self.executable.uri.clone(),
                                            action:
                                                UniformResourceWriterAction::CapturedExecutableNonZeroExit(
                                                    exit_status,
                                                    stderr,
                                                    captured_executable_diags
                                                ),
                                        }
                                    }
                                }
                                Err(err) => UniformResourceWriterResult {
                                    uri: self.executable.uri.clone(),
                                    action: UniformResourceWriterAction::CapturableExecError(err),
                                },
                            }
                        }
                        None => UniformResourceWriterResult {
                            uri: self.executable.uri.clone(),
                            action: UniformResourceWriterAction::CapturableExecNoCaptureSupplier(),
                        },
                    }
                }
                CapturableExecutable::RequestedButNoNature(re) => UniformResourceWriterResult {
                    uri: self.executable.uri.clone(),
                    action: UniformResourceWriterAction::CapturableExecNoNature(re.clone()),
                },
                CapturableExecutable::RequestedButNotExecutable => UniformResourceWriterResult {
                    uri: self.executable.uri.clone(),
                    action: UniformResourceWriterAction::CapturableExecNotExecutable(),
                },
            },
            None => UniformResourceWriterResult {
                uri: self.executable.uri.clone(),
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
                        let fm_attrs_value = serde_json::json!({
                            "frontMatter": fm_raw.unwrap(),
                            "body": fm_body,
                            "attrs": fm_json.clone().unwrap()
                        });
                        fm_attrs = Some(serde_json::to_string_pretty(&fm_attrs_value).unwrap());
                    }
                    let uri = self.resource.uri.to_string();
                    match urw_state.ins_ur_stmt.query_row(
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

impl UniformResourceWriter<ContentResource> for SoftwarePackageDxResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for SvgResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for TestAnythingResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for TomlResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResourceWriter<ContentResource> for YamlResource<ContentResource> {
    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        self.insert_text(urw_state, &self.resource, entry)
    }
}

impl UniformResource<ContentResource> {
    fn _uri(&self) -> &str {
        match self {
            UniformResource::CapturableExec(capturable) => capturable.executable.uri.as_str(),
            UniformResource::Html(html) => html.resource.uri.as_str(),
            UniformResource::Json(json) => json.resource.uri.as_str(),
            UniformResource::Image(img) => img.resource.uri.as_str(),
            UniformResource::Markdown(md) => md.resource.uri.as_str(),
            UniformResource::PlainText(txt) => txt.resource.uri.as_str(),
            UniformResource::SpdxJson(spdx) => spdx.resource.uri.as_str(),
            UniformResource::Svg(svg) => svg.resource.uri.as_str(),
            UniformResource::Tap(tap) => tap.resource.uri.as_str(),
            UniformResource::Toml(toml) => toml.resource.uri.as_str(),
            UniformResource::Yaml(yaml) => yaml.resource.uri.as_str(),
            UniformResource::Unknown(unknown, _) => unknown.uri.as_str(),
        }
    }

    fn insert(
        &self,
        urw_state: &mut UniformResourceWriterState<'_, '_>,
        entry: &mut UniformResourceWriterEntry,
    ) -> UniformResourceWriterResult {
        match self {
            UniformResource::CapturableExec(capturable) => capturable.insert(urw_state, entry),
            UniformResource::Html(html) => html.insert(urw_state, entry),
            UniformResource::Json(json) => json.insert(urw_state, entry),
            UniformResource::Image(img) => img.insert(urw_state, entry),
            UniformResource::Markdown(md) => md.insert(urw_state, entry),
            UniformResource::PlainText(txt) => txt.insert(urw_state, entry),
            UniformResource::SpdxJson(spdx) => spdx.insert(urw_state, entry),
            UniformResource::Svg(svg) => svg.insert(urw_state, entry),
            UniformResource::Tap(tap) => tap.insert(urw_state, entry),
            UniformResource::Toml(toml) => toml.insert(urw_state, entry),
            UniformResource::Yaml(yaml) => yaml.insert(urw_state, entry),
            UniformResource::Unknown(unknown, tried_alternate_nature) => {
                if let Some(tried_alternate_nature) = tried_alternate_nature {
                    entry.tried_alternate_nature = Some(tried_alternate_nature.clone());
                }
                unknown.insert(urw_state, entry)
            }
        }
    }
}

// TODO: Allow per file type / MIME / extension / nature configuration
// such as compression for certain types of files but as-is for other
// types;

#[derive(Serialize, Deserialize)]
pub struct IngestBehavior {
    pub root_fs_paths: Vec<String>,

    #[serde(with = "serde_regex")]
    pub ignore_fs_entry_regexs: Vec<regex::Regex>,

    #[serde(with = "serde_regex")]
    pub ingest_content_fs_entry_regexs: Vec<regex::Regex>,

    #[serde(with = "serde_regex")]
    pub compute_digests_fs_entry_regexs: Vec<regex::Regex>,

    #[serde(with = "serde_regex")]
    pub capturable_executables_fs_entry_regexs: Vec<regex::Regex>,

    #[serde(with = "serde_regex")]
    pub captured_exec_sql_fs_entry_regexs: Vec<regex::Regex>,

    pub code_notebooks_searched: Vec<String>,
    pub code_notebook_cells_searched: Vec<String>,
    pub nature_bind: HashMap<String, String>,
}

impl IngestBehavior {
    pub fn new(
        device_id: &String,
        fsw_args: &super::IngestArgs,
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
                        "[IngestBehavior.new] unable to read behavior '{}' from {} behavior table",
                        behavior_name, fsw_args.state_db_fs_path
                    )
                })?;
            let fswb = IngestBehavior::from_json(&fswb_json).with_context(|| {
                format!(
                    "[IngestBehavior.new] unable to deserialize behavior {} in {}",
                    fswb_json, fsw_args.state_db_fs_path
                )
            })?;
            Ok((fswb, Some(fswb_behavior_id)))
        } else {
            Ok((IngestBehavior::from_ingest_args(fsw_args), None))
        }
    }

    pub fn from_ingest_args(args: &super::IngestArgs) -> Self {
        let mut nature_bind: HashMap<String, String> =
            if let Some(supplied_binds) = &args.nature_bind {
                supplied_binds.clone()
            } else {
                HashMap::new()
            };
        if !nature_bind.contains_key("text") {
            nature_bind.insert("text".to_string(), "text/plain".to_string());
        }
        if !nature_bind.contains_key("yaml") {
            nature_bind.insert("yaml".to_string(), "application/yaml".to_string());
        }

        // the names in `args` are convenient for CLI usage but the struct
        // field names in IngestBehavior should be longer and more descriptive
        // since IngestBehavior is stored as activity in the database.
        IngestBehavior {
            root_fs_paths: args.root_fs_path.clone(),
            ingest_content_fs_entry_regexs: args.surveil_fs_content.clone(),
            compute_digests_fs_entry_regexs: args.compute_fs_content_digests.clone(),
            ignore_fs_entry_regexs: args.ignore_fs_entry.clone(),
            capturable_executables_fs_entry_regexs: args.capture_fs_exec.clone(),
            captured_exec_sql_fs_entry_regexs: args.captured_fs_exec_sql.clone(),
            code_notebooks_searched: args.notebook.clone(),
            code_notebook_cells_searched: args.cell.clone(),
            nature_bind,
        }
    }

    pub fn from_json(json_text: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json_text)
    }

    pub fn persistable_json_text(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    pub fn add_ignore_exact(&mut self, pattern: &str) {
        self.ignore_fs_entry_regexs
            .push(regex::Regex::new(format!("^{}$", regex::escape(pattern)).as_str()).unwrap());
    }

    pub fn code_notebook_cells(&mut self, conn: &Connection) {
        match select_notebooks_and_cells(
            conn,
            &self.code_notebooks_searched,
            &self.code_notebook_cells_searched,
        ) {
            Ok(matched) => {
                for row in matched {
                    let (notebook, kernel, cell, _code) = row;
                    println!("-- {notebook}::{cell} ({kernel})");
                }
            }
            Err(err) => println!("IngestBehavior code_notebook_cells error: {}", err),
        }
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
                    "[IngestBehavior.save] unable to save behavior '{}'",
                    behavior_name
                )
            })?;
        Ok(behavior_id)
    }
}

pub fn ingest(cli: &super::Cli, fsw_args: &super::IngestArgs) -> Result<String> {
    let db_fs_path = &fsw_args.state_db_fs_path;

    if cli.debug > 0 {
        println!("Surveillance State DB: {}", db_fs_path);
    }

    let mut conn = Connection::open(db_fs_path)
        .with_context(|| format!("[ingest] SQLite database {}", db_fs_path))?;

    prepare_conn(&conn)
        .with_context(|| format!("[ingest] prepare SQLite connection for {}", db_fs_path))?;

    // putting everything inside a transaction improves performance significantly
    let tx = conn
        .transaction()
        .with_context(|| format!("[ingest] SQLite transaction in {}", db_fs_path))?;

    execute_migrations(&tx, "ingest")
        .with_context(|| format!("[ingest] execute_migrations in {}", db_fs_path))?;

    // TODO: add the executed files into the behaviors or other activity log!?
    let executed = execute_globs_batch(
        &tx,
        &execute_globs_batch_cfse(&fsw_args.state_db_init_sql),
        &[".".to_string()],
        "ingest",
    )
    .with_context(|| {
        format!(
            "[ingest] execute_globs_batch {} in {}",
            fsw_args.state_db_init_sql.join(", "),
            db_fs_path
        )
    })?;
    if cli.debug > 0 {
        println!("Executed init SQL: {}", executed.join(", "))
    }

    // insert the device or, if it exists, get its current ID and name
    let (device_id, device_name) = upserted_device(&tx, &crate::DEVICE).with_context(|| {
        format!(
            "[ingest] upserted_device {} in {}",
            crate::DEVICE.name,
            db_fs_path
        )
    })?;

    if cli.debug > 0 {
        println!("Device: {device_name} ({device_id})");
    }

    // the ulid() function we're using below is not built into SQLite, we define
    // it in persist::prepare_conn.

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
    // in ins_ur_stmt the `DO UPDATE SET size_bytes = EXCLUDED.size_bytes` is a workaround to force uniform_resource_id when the row already exists
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

    let (mut fswb, mut behavior_id) = IngestBehavior::new(&device_id, fsw_args, &tx)
        .with_context(|| format!("[ingest] behavior issue {}", db_fs_path))?;

    if !fsw_args.include_state_db_in_ingestion {
        let canonical_db_fs_path = std::fs::canonicalize(std::path::Path::new(&db_fs_path))
            .with_context(|| format!("[ingest] unable to canonicalize in {}", db_fs_path))?;
        let canonical_db_fs_path = canonical_db_fs_path.to_string_lossy().to_string();
        let mut wal_path = std::path::PathBuf::from(&canonical_db_fs_path);
        let mut db_journal_path = std::path::PathBuf::from(&canonical_db_fs_path);
        wal_path.set_extension("wal");
        db_journal_path.set_extension("db-journal");
        fswb.add_ignore_exact(canonical_db_fs_path.as_str());
        fswb.add_ignore_exact(wal_path.to_string_lossy().to_string().as_str());
        fswb.add_ignore_exact(db_journal_path.to_string_lossy().to_string().as_str());
    }

    if let Some(save_behavior_name) = &fsw_args.save_behavior {
        let saved_bid = fswb
            .save(&tx, &device_id, save_behavior_name)
            .with_context(|| format!("[ingest] saving {} in {}", save_behavior_name, db_fs_path))?;
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
                "[ingest] inserting UR walk session using {} in {}",
                INS_UR_INGEST_SESSION_SQL, db_fs_path
            )
        })?;
    if cli.debug > 0 {
        println!("Walk Session: {ingest_session_id}");
    }

    // We don't use an ORM and just use raw Rusqlite for highest performance.
    // Use a scope to ensure all prepared SQL statements, which borrow `tx`` are dropped before committing the transaction.
    {
        let env_current_dir = std::env::current_dir()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let mut ins_ur_isfsp_stmt = tx.prepare(INS_UR_ISFSP_SQL).with_context(|| {
            format!(
                "[ingest] unable to create `ins_ur_isfsp_stmt` SQL {} in {}",
                INS_UR_ISFSP_SQL, db_fs_path
            )
        })?;
        let mut ins_ur_stmt = tx.prepare(INS_UR_SQL).with_context(|| {
            format!(
                "[ingest] unable to create `ins_ur_stmt` SQL {} in {}",
                INS_UR_SQL, db_fs_path
            )
        })?;
        let mut ins_ur_transform_stmt = tx.prepare(INS_UR_TRANSFORM_SQL).with_context(|| {
            format!(
                "[ingest] unable to create `ins_ur_transform_stmt` SQL {} in {}",
                INS_UR_TRANSFORM_SQL, db_fs_path
            )
        })?;
        let mut ins_ur_isfsp_entry_stmt =
            tx.prepare(INS_UR_ISFSP_ENTRY_SQL).with_context(|| {
                format!(
                    "[ingest] unable to create `ins_ur_isfsp_entry_stmt` SQL {} in {}",
                    INS_UR_ISFSP_ENTRY_SQL, db_fs_path
                )
            })?;

        fswb.code_notebook_cells(&tx);

        for root_path in &fswb.root_fs_paths {
            let canonical_path_buf = std::fs::canonicalize(std::path::Path::new(&root_path))
                .with_context(|| {
                    format!(
                        "[ingest] unable to canonicalize {} in {}",
                        root_path, db_fs_path
                    )
                })?;
            let canonical_path = canonical_path_buf.into_os_string().into_string().unwrap();

            let ins_ur_wsp_params = params![ingest_session_id, canonical_path];
            let ingest_fs_path_id: String = ins_ur_isfsp_stmt
                .query_row(ins_ur_wsp_params, |row| row.get(0))
                .with_context(|| {
                    format!(
                        "[ingest] ins_ur_wsp_stmt {} with {} in {}",
                        INS_UR_ISFSP_SQL, "TODO: ins_ur_wsp_params.join()", db_fs_path
                    )
                })?;
            if cli.debug > 0 {
                println!("  Walk Session Path: {root_path} ({ingest_fs_path_id})");
            }

            let rp: Vec<String> = vec![canonical_path.clone()];
            let walker = FileSysResourcesWalker::new(
                &rp,
                &fswb.ignore_fs_entry_regexs,
                &fswb.ingest_content_fs_entry_regexs,
                &fswb.capturable_executables_fs_entry_regexs,
                &fswb.captured_exec_sql_fs_entry_regexs,
                &fswb.nature_bind,
            )
            .with_context(|| {
                format!(
                    "[ingest] unable to walker for {} in {}",
                    canonical_path, db_fs_path
                )
            })?;

            let mut urw_state = UniformResourceWriterState {
                ingest_args: fsw_args,
                ingest_behavior: &fswb,
                env_current_dir: &env_current_dir,
                device_id: &device_id,
                ingest_session_id: &ingest_session_id,
                ingest_fs_path_id: &ingest_fs_path_id,
                fsr_walker: &walker,
                ins_ur_stmt: &mut ins_ur_stmt,
                _ins_ur_transform_stmt: &mut ins_ur_transform_stmt,
            };

            for resource_result in walker.walk_resources_iter() {
                match resource_result {
                    Ok((entry, resource)) => {
                        let mut urw_entry = UniformResourceWriterEntry {
                            dir_entry: &entry,
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
                                match ins_ur_isfsp_entry_stmt.execute(params![
                                    ingest_session_id,
                                    ingest_fs_path_id,
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
                                    ur_status,
                                    ur_diagnostics,
                                    captured_exec_diags
                                ]) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        eprintln!( "[ingest] unable to insert UR walk session path file system entry for {} in {}: {} ({})",
                                        &inserted.uri, db_fs_path, err, INS_UR_ISFSP_ENTRY_SQL
                                        )
                                    }
                                }
                            }
                            None => {
                                eprintln!(
                                    "[ingest] error extracting path info for {} in {}",
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

    match tx.execute(INS_UR_INGEST_SESSION_FINISH_SQL, params![ingest_session_id]) {
        Ok(_) => {}
        Err(err) => {
            eprintln!(
                "[ingest] unable to execute SQL {} in {}: {}",
                INS_UR_INGEST_SESSION_FINISH_SQL, db_fs_path, err
            )
        }
    }
    // putting everything inside a transaction improves performance significantly
    tx.commit()
        .with_context(|| format!("[ingest] unable to perform final commit in {}", db_fs_path))?;

    Ok(ingest_session_id)
}
