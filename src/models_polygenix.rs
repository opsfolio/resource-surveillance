#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct device {
    device_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    name: String, // 'string' maps directly to Rust type
    state: String, // uknown type 'string::json', mapping to String by default
    boundary: String, // 'string' maps directly to Rust type
    segmentation: Some(String), // uknown type 'string::json', mapping to String by default
    state_sysinfo: Some(String), // uknown type 'string::json', mapping to String by default
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct behavior {
    behavior_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    device_id: String, // 'string' maps directly to Rust type
    behavior_name: String, // 'string' maps directly to Rust type
    behavior_conf_json: String, // uknown type 'string::json', mapping to String by default
    assurance_schema_id: Some(String), // 'string' maps directly to Rust type
    governance: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_resource_path_match_rule {
    ur_ingest_resource_path_match_rule_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    namespace: String, // 'string' maps directly to Rust type
    regex: String, // 'string' maps directly to Rust type
    flags: String, // 'string' maps directly to Rust type
    nature: Some(String), // 'string' maps directly to Rust type
    priority: Some(String), // 'string' maps directly to Rust type
    description: Some(String), // 'string' maps directly to Rust type
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_resource_path_rewrite_rule {
    ur_ingest_resource_path_rewrite_rule_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    namespace: String, // 'string' maps directly to Rust type
    regex: String, // 'string' maps directly to Rust type
    replace: String, // 'string' maps directly to Rust type
    priority: Some(String), // 'string' maps directly to Rust type
    description: Some(String), // 'string' maps directly to Rust type
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session {
    ur_ingest_session_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    device_id: String, // 'string' maps directly to Rust type
    behavior_id: Some(String), // 'string' maps directly to Rust type
    behavior_json: Some(String), // uknown type 'string::json', mapping to String by default
    ingest_started_at: String, // uknown type 'timestamp', mapping to String by default
    ingest_finished_at: Some(String), // uknown type 'timestamp', mapping to String by default
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session_fs_path {
    ur_ingest_session_fs_path_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    ingest_session_id: String, // 'string' maps directly to Rust type
    root_path: String, // 'string' maps directly to Rust type
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct uniform_resource {
    uniform_resource_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    device_id: String, // 'string' maps directly to Rust type
    ingest_session_id: String, // 'string' maps directly to Rust type
    ingest_fs_path_id: Some(String), // 'string' maps directly to Rust type
    uri: String, // 'string' maps directly to Rust type
    content_digest: String, // 'string' maps directly to Rust type
    content: Some(Vec<u8>), // 'blob' maps directly to Rust type
    nature: Some(String), // 'string' maps directly to Rust type
    size_bytes: Some(i64), // 'integer' maps directly to Rust type
    last_modified_at: Some(String), // uknown type 'timestamp', mapping to String by default
    content_fm_body_attrs: Some(String), // uknown type 'string::json', mapping to String by default
    frontmatter: Some(String), // uknown type 'string::json', mapping to String by default
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct uniform_resource_transform {
    uniform_resource_transform_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    uniform_resource_id: String, // 'string' maps directly to Rust type
    uri: String, // 'string' maps directly to Rust type
    content_digest: String, // 'string' maps directly to Rust type
    content: Some(Vec<u8>), // 'blob' maps directly to Rust type
    nature: Some(String), // 'string' maps directly to Rust type
    size_bytes: Some(i64), // 'integer' maps directly to Rust type
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session_fs_path_entry {
    ur_ingest_session_fs_path_entry_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    ingest_session_id: String, // 'string' maps directly to Rust type
    ingest_fs_path_id: String, // 'string' maps directly to Rust type
    uniform_resource_id: Some(String), // 'string' maps directly to Rust type
    file_path_abs: String, // 'string' maps directly to Rust type
    file_path_rel_parent: String, // 'string' maps directly to Rust type
    file_path_rel: String, // 'string' maps directly to Rust type
    file_basename: String, // 'string' maps directly to Rust type
    file_extn: Some(String), // 'string' maps directly to Rust type
    captured_executable: Some(String), // uknown type 'string::json', mapping to String by default
    ur_status: Some(String), // 'string' maps directly to Rust type
    ur_diagnostics: Some(String), // uknown type 'string::json', mapping to String by default
    ur_transformations: Some(String), // uknown type 'string::json', mapping to String by default
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ur_ingest_session_task {
    ur_ingest_session_task_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    ingest_session_id: String, // 'string' maps directly to Rust type
    uniform_resource_id: Some(String), // 'string' maps directly to Rust type
    captured_executable: String, // uknown type 'string::json', mapping to String by default
    ur_status: Some(String), // 'string' maps directly to Rust type
    ur_diagnostics: Some(String), // uknown type 'string::json', mapping to String by default
    ur_transformations: Some(String), // uknown type 'string::json', mapping to String by default
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct assurance_schema {
    assurance_schema_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    assurance_type: String, // 'string' maps directly to Rust type
    code: String, // 'string' maps directly to Rust type
    code_json: Some(String), // uknown type 'string::json', mapping to String by default
    governance: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct code_notebook_kernel {
    code_notebook_kernel_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    kernel_name: String, // 'string' maps directly to Rust type
    description: Some(String), // 'string' maps directly to Rust type
    mime_type: Some(String), // 'string' maps directly to Rust type
    file_extn: Some(String), // 'string' maps directly to Rust type
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    governance: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct code_notebook_cell {
    code_notebook_cell_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    notebook_kernel_id: String, // 'string' maps directly to Rust type
    notebook_name: String, // 'string' maps directly to Rust type
    cell_name: String, // 'string' maps directly to Rust type
    cell_governance: Some(String), // uknown type 'string::json', mapping to String by default
    interpretable_code: String, // 'string' maps directly to Rust type
    interpretable_code_hash: String, // 'string' maps directly to Rust type
    description: Some(String), // 'string' maps directly to Rust type
    arguments: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}

#[derive(Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct code_notebook_state {
    code_notebook_state_id: String, // PRIMARY KEY ('string' maps directly to Rust type)
    code_notebook_cell_id: String, // 'string' maps directly to Rust type
    from_state: String, // 'string' maps directly to Rust type
    to_state: String, // 'string' maps directly to Rust type
    transition_result: Some(String), // uknown type 'string::json', mapping to String by default
    transition_reason: Some(String), // 'string' maps directly to Rust type
    transitioned_at: Some(String), // uknown type 'timestamp', mapping to String by default
    elaboration: Some(String), // uknown type 'string::json', mapping to String by default
    created_at: Some(String), // uknown type 'timestamp', mapping to String by default
    created_by: Some(String), // 'string' maps directly to Rust type
    updated_at: Some(String), // uknown type 'timestamp', mapping to String by default
    updated_by: Some(String), // 'string' maps directly to Rust type
    deleted_at: Some(String), // uknown type 'timestamp', mapping to String by default
    deleted_by: Some(String), // 'string' maps directly to Rust type
    activity_log: Some(String), // uknown type 'jsonb', mapping to String by default
}
