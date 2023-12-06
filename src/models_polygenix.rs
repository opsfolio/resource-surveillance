#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct device {
    device_id: String, // PRIMARY KEY
    name: String,
    state: String,
    boundary: String,
    segmentation: Some(String),
    state_sysinfo: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct behavior {
    behavior_id: String, // PRIMARY KEY
    device_id: String,
    behavior_name: String,
    behavior_conf_json: String,
    assurance_schema_id: Some(String),
    governance: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_resource_path_match_rule {
    ur_ingest_resource_path_match_rule_id: String, // PRIMARY KEY
    namespace: String,
    regex: String,
    flags: String,
    nature: Some(String),
    priority: Some(String),
    description: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_resource_path_rewrite_rule {
    ur_ingest_resource_path_rewrite_rule_id: String, // PRIMARY KEY
    namespace: String,
    regex: String,
    replace: String,
    priority: Some(String),
    description: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session {
    ur_ingest_session_id: String, // PRIMARY KEY
    device_id: String,
    behavior_id: Some(String),
    behavior_json: Some(String),
    ingest_started_at: String,
    ingest_finished_at: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session_fs_path {
    ur_ingest_session_fs_path_id: String, // PRIMARY KEY
    ingest_session_id: String,
    root_path: String,
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct uniform_resource {
    uniform_resource_id: String, // PRIMARY KEY
    device_id: String,
    ingest_session_id: String,
    ingest_fs_path_id: Some(String),
    uri: String,
    content_digest: String,
    content: Some(Vec<u8>),
    nature: Some(String),
    size_bytes: Some(i64),
    last_modified_at: Some(String),
    content_fm_body_attrs: Some(String),
    frontmatter: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct uniform_resource_transform {
    uniform_resource_transform_id: String, // PRIMARY KEY
    uniform_resource_id: String,
    uri: String,
    content_digest: String,
    content: Some(Vec<u8>),
    nature: Some(String),
    size_bytes: Some(i64),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session_fs_path_entry {
    ur_ingest_session_fs_path_entry_id: String, // PRIMARY KEY
    ingest_session_id: String,
    ingest_fs_path_id: String,
    uniform_resource_id: Some(String),
    file_path_abs: String,
    file_path_rel_parent: String,
    file_path_rel: String,
    file_basename: String,
    file_extn: Some(String),
    captured_executable: Some(String),
    ur_status: Some(String),
    ur_diagnostics: Some(String),
    ur_transformations: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session_task {
    ur_ingest_session_task_id: String, // PRIMARY KEY
    ingest_session_id: String,
    uniform_resource_id: Some(String),
    captured_executable: String,
    ur_status: Some(String),
    ur_diagnostics: Some(String),
    ur_transformations: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct assurance_schema {
    assurance_schema_id: String, // PRIMARY KEY
    assurance_type: String,
    code: String,
    code_json: Some(String),
    governance: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct code_notebook_kernel {
    code_notebook_kernel_id: String, // PRIMARY KEY
    kernel_name: String,
    description: Some(String),
    mime_type: Some(String),
    file_extn: Some(String),
    elaboration: Some(String),
    governance: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct code_notebook_cell {
    code_notebook_cell_id: String, // PRIMARY KEY
    notebook_kernel_id: String,
    notebook_name: String,
    cell_name: String,
    cell_governance: Some(String),
    interpretable_code: String,
    interpretable_code_hash: String,
    description: Some(String),
    arguments: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct code_notebook_state {
    code_notebook_state_id: String, // PRIMARY KEY
    code_notebook_cell_id: String,
    from_state: String,
    to_state: String,
    transition_result: Some(String),
    transition_reason: Some(String),
    transitioned_at: Some(String),
    elaboration: Some(String),
    created_at: Some(String),
    created_by: Some(String),
    updated_at: Some(String),
    updated_by: Some(String),
    deleted_at: Some(String),
    deleted_by: Some(String),
    activity_log: Some(String),
}
