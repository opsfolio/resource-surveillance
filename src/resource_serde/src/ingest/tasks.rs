use std::collections::HashMap;

use super::{
    insert_uniform_resource, IngestContext, IngestTasksBehavior, UniformResourceWriterAction,
    UniformResourceWriterEntry, UniformResourceWriterState, INS_UR_INGEST_SESSION_FINISH_SQL,
    INS_UR_INGEST_SESSION_SQL, INS_UR_IS_TASK_SQL,
};
use crate::cmd::IngestTasksArgs;
use anyhow::{Context, Result};
use rusqlite::params;
use serde_json::json;
use tracing::debug;
use tracing::error;

use crate::persist::*;
use resource::*;

// #[autometrics]
pub fn ingest_tasks(debug: u8, ingest_args: &IngestTasksArgs) -> Result<String> {
    let mut dbc = DbConn::new(&ingest_args.state_db_fs_path, debug).with_context(|| {
        format!(
            "[ingest_tasks] SQLite transaction in {}",
            ingest_args.state_db_fs_path
        )
    })?;
    let db_fs_path = dbc.db_fs_path.clone();

    // putting everything inside a transaction improves performance significantly
    let tx = dbc.init(Some(&ingest_args.state_db_init_sql))?;
    let (device_id, _device_name) = upserted_device(&tx, &common::DEVICE).with_context(|| {
        format!(
            "[ingest_tasks] upserted_device {} in {}",
            common::DEVICE.name,
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

    debug!("Walk Session: {ingest_session_id}");

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

                    debug!("{:?}", urw_entry.path);

                    let inserted =
                        insert_uniform_resource(&resource, &mut urw_state, &mut urw_entry);
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
                            error!( "[ingest_tasks] unable to insert UR task entry for {} in {}: {} ({})",
                            &inserted.uri, db_fs_path, err, INS_UR_IS_TASK_SQL
                            )
                        }
                    }
                }
                Err(e) => {
                    error!("Error processing a ingest_tasks resource: {}", e);
                }
            }
        }
    }

    match tx.execute(INS_UR_INGEST_SESSION_FINISH_SQL, params![ingest_session_id]) {
        Ok(_) => {}
        Err(err) => {
            error!(
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
