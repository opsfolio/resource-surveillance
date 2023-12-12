/*
const DEVICE: &str = "device";
const BEHAVIOR: &str = "behavior";
const UR_INGEST_RESOURCE_PATH_MATCH_RULE: &str = "ur_ingest_resource_path_match_rule";
const UR_INGEST_RESOURCE_PATH_REWRITE_RULE: &str = "ur_ingest_resource_path_rewrite_rule";
const UR_INGEST_SESSION: &str = "ur_ingest_session";
const UR_INGEST_SESSION_FS_PATH: &str = "ur_ingest_session_fs_path";
const UNIFORM_RESOURCE: &str = "uniform_resource";
const UNIFORM_RESOURCE_TRANSFORM: &str = "uniform_resource_transform";
const UR_INGEST_SESSION_FS_PATH_ENTRY: &str = "ur_ingest_session_fs_path_entry";
const UR_INGEST_SESSION_TASK: &str = "ur_ingest_session_task";
const ASSURANCE_SCHEMA: &str = "assurance_schema";
const CODE_NOTEBOOK_KERNEL: &str = "code_notebook_kernel";
const CODE_NOTEBOOK_CELL: &str = "code_notebook_cell";
const CODE_NOTEBOOK_STATE: &str = "code_notebook_state";
*/

// `device` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Device {
    device_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    name: String, // 'string' maps directly to Rust type
    state: String, // uknown type 'string::json', mapping to String by default
    boundary: String, // 'string' maps directly to Rust type
    segmentation: Option<String>, // uknown type 'string::json', mapping to String by default
    state_sysinfo: Option<String>, // uknown type 'string::json', mapping to String by default
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
    behaviors: Vec<Behavior>, // `behavior` belongsTo collection
    ur_ingest_sessions: Vec<UrIngestSession>, // `ur_ingest_session` belongsTo collection
    uniform_resources: Vec<UniformResource>, // `uniform_resource` belongsTo collection
}

// `behavior` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Behavior {
    behavior_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    device_id: String, // 'string' maps directly to Rust type
    behavior_name: String, // 'string' maps directly to Rust type
    behavior_conf_json: String, // uknown type 'string::json', mapping to String by default
    assurance_schema_id: Option<String>, // 'string' maps directly to Rust type
    governance: Option<String>, // uknown type 'string::json', mapping to String by default
    ur_ingest_sessions: Vec<UrIngestSession>, // `ur_ingest_session` belongsTo collection
}

// `ur_ingest_resource_path_match_rule` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UrIngestResourcePathMatchRule {
    ur_ingest_resource_path_match_rule_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    namespace: String, // 'string' maps directly to Rust type
    regex: String, // 'string' maps directly to Rust type
    flags: String, // 'string' maps directly to Rust type
    nature: Option<String>, // 'string' maps directly to Rust type
    priority: Option<String>, // 'string' maps directly to Rust type
    description: Option<String>, // 'string' maps directly to Rust type
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `ur_ingest_resource_path_rewrite_rule` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UrIngestResourcePathRewriteRule {
    ur_ingest_resource_path_rewrite_rule_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    namespace: String, // 'string' maps directly to Rust type
    regex: String, // 'string' maps directly to Rust type
    replace: String, // 'string' maps directly to Rust type
    priority: Option<String>, // 'string' maps directly to Rust type
    description: Option<String>, // 'string' maps directly to Rust type
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `ur_ingest_session` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UrIngestSession {
    ur_ingest_session_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    device_id: String, // 'string' maps directly to Rust type
    behavior_id: Option<String>, // 'string' maps directly to Rust type
    behavior_json: Option<String>, // uknown type 'string::json', mapping to String by default
    ingest_started_at: String, // uknown type 'TIMESTAMPTZ', mapping to String by default
    ingest_finished_at: Option<String>, // uknown type 'TIMESTAMPTZ', mapping to String by default
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
    ur_ingest_session_fs_paths: Vec<UrIngestSessionFsPath>, // `ur_ingest_session_fs_path` belongsTo collection
    uniform_resources: Vec<UniformResource>, // `uniform_resource` belongsTo collection
    ur_ingest_session_fs_path_entrys: Vec<UrIngestSessionFsPathEntry>, // `ur_ingest_session_fs_path_entry` belongsTo collection
}

// `ur_ingest_session_fs_path` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UrIngestSessionFsPath {
    ur_ingest_session_fs_path_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    ingest_session_id: String, // 'string' maps directly to Rust type
    root_path: String, // 'string' maps directly to Rust type
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
    ur_ingest_session_fs_path_entrys: Vec<UrIngestSessionFsPathEntry>, // `ur_ingest_session_fs_path_entry` belongsTo collection
}

// `uniform_resource` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UniformResource {
    uniform_resource_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    device_id: String, // 'string' maps directly to Rust type
    ingest_session_id: String, // 'string' maps directly to Rust type
    ingest_fs_path_id: Option<String>, // 'string' maps directly to Rust type
    uri: String, // 'string' maps directly to Rust type
    content_digest: String, // 'string' maps directly to Rust type
    content: Option<Vec<u8>>, // 'blob' maps directly to Rust type
    nature: Option<String>, // 'string' maps directly to Rust type
    size_bytes: Option<i64>, // 'integer' maps directly to Rust type
    last_modified_at: Option<String>, // uknown type 'TIMESTAMPTZ', mapping to String by default
    content_fm_body_attrs: Option<String>, // uknown type 'string::json', mapping to String by default
    frontmatter: Option<String>, // uknown type 'string::json', mapping to String by default
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
    uniform_resource_transforms: Vec<UniformResourceTransform>, // `uniform_resource_transform` belongsTo collection
}

// `uniform_resource_transform` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UniformResourceTransform {
    uniform_resource_transform_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    uniform_resource_id: String, // 'string' maps directly to Rust type
    uri: String, // 'string' maps directly to Rust type
    content_digest: String, // 'string' maps directly to Rust type
    content: Option<Vec<u8>>, // 'blob' maps directly to Rust type
    nature: Option<String>, // 'string' maps directly to Rust type
    size_bytes: Option<i64>, // 'integer' maps directly to Rust type
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `ur_ingest_session_fs_path_entry` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UrIngestSessionFsPathEntry {
    ur_ingest_session_fs_path_entry_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    ingest_session_id: String, // 'string' maps directly to Rust type
    ingest_fs_path_id: String, // 'string' maps directly to Rust type
    uniform_resource_id: Option<String>, // 'string' maps directly to Rust type
    file_path_abs: String, // 'string' maps directly to Rust type
    file_path_rel_parent: String, // 'string' maps directly to Rust type
    file_path_rel: String, // 'string' maps directly to Rust type
    file_basename: String, // 'string' maps directly to Rust type
    file_extn: Option<String>, // 'string' maps directly to Rust type
    captured_executable: Option<String>, // uknown type 'string::json', mapping to String by default
    ur_status: Option<String>, // 'string' maps directly to Rust type
    ur_diagnostics: Option<String>, // uknown type 'string::json', mapping to String by default
    ur_transformations: Option<String>, // uknown type 'string::json', mapping to String by default
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `ur_ingest_session_task` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct UrIngestSessionTask {
    ur_ingest_session_task_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    ingest_session_id: String, // 'string' maps directly to Rust type
    uniform_resource_id: Option<String>, // 'string' maps directly to Rust type
    captured_executable: String, // uknown type 'string::json', mapping to String by default
    ur_status: Option<String>, // 'string' maps directly to Rust type
    ur_diagnostics: Option<String>, // uknown type 'string::json', mapping to String by default
    ur_transformations: Option<String>, // uknown type 'string::json', mapping to String by default
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `assurance_schema` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AssuranceSchema {
    assurance_schema_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    assurance_type: String, // 'string' maps directly to Rust type
    code: String, // 'string' maps directly to Rust type
    code_json: Option<String>, // uknown type 'string::json', mapping to String by default
    governance: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `code_notebook_kernel` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CodeNotebookKernel {
    code_notebook_kernel_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    kernel_name: String, // 'string' maps directly to Rust type
    description: Option<String>, // 'string' maps directly to Rust type
    mime_type: Option<String>, // 'string' maps directly to Rust type
    file_extn: Option<String>, // 'string' maps directly to Rust type
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
    governance: Option<String>, // uknown type 'string::json', mapping to String by default
    code_notebook_cells: Vec<CodeNotebookCell>, // `code_notebook_cell` belongsTo collection
}

// `code_notebook_cell` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CodeNotebookCell {
    code_notebook_cell_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    notebook_kernel_id: String, // 'string' maps directly to Rust type
    notebook_name: String, // 'string' maps directly to Rust type
    cell_name: String, // 'string' maps directly to Rust type
    cell_governance: Option<String>, // uknown type 'string::json', mapping to String by default
    interpretable_code: String, // 'string' maps directly to Rust type
    interpretable_code_hash: String, // 'string' maps directly to Rust type
    description: Option<String>, // 'string' maps directly to Rust type
    arguments: Option<String>, // uknown type 'string::json', mapping to String by default
}

// `code_notebook_state` table
#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CodeNotebookState {
    code_notebook_state_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    code_notebook_cell_id: String, // 'string' maps directly to Rust type
    from_state: String, // 'string' maps directly to Rust type
    to_state: String, // 'string' maps directly to Rust type
    transition_result: Option<String>, // uknown type 'string::json', mapping to String by default
    transition_reason: Option<String>, // 'string' maps directly to Rust type
    transitioned_at: Option<String>, // uknown type 'TIMESTAMPTZ', mapping to String by default
    elaboration: Option<String>, // uknown type 'string::json', mapping to String by default
}
