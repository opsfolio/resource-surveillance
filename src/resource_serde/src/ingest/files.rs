use crate::{
    cmd::IngestFilesArgs,
    ingest::{
        insert_uniform_resource, upserted_device, DbConn, IngestContext, IngestFilesBehavior,
        UniformResourceWriterAction, UniformResourceWriterEntry, UniformResourceWriterState,
        INS_UR_INGEST_SESSION_FINISH_SQL, INS_UR_INGEST_SESSION_SQL, INS_UR_ISFSP_ENTRY_SQL,
        INS_UR_ISFSP_SQL,
    },
};
use anyhow::{Context, Result};
use resource::{extract_path_info, ResourcesCollection, UriNatureSupplier};
use rusqlite::params;
use serde_json::json;
use tracing::{debug, error};

pub fn ingest_files(debug: u8, ingest_args: &IngestFilesArgs) -> Result<String> {
    let mut dbc = DbConn::new(&ingest_args.state_db_fs_path, debug).with_context(|| {
        format!(
            "[ingest_files] SQLite transaction in {}",
            ingest_args.state_db_fs_path
        )
    })?;
    let db_fs_path = dbc.db_fs_path.clone();

    // putting everything inside a transaction improves performance significantly
    let tx = dbc.init(Some(&ingest_args.state_db_init_sql))?;
    let (device_id, _device_name) = upserted_device(&tx, &common::DEVICE).with_context(|| {
        format!(
            "[ingest_files] upserted_device {} in {}",
            common::DEVICE.name,
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

        debug!("Saved behavior: {} ({})", save_behavior_name, saved_bid);
        behavior_id = Some(saved_bid);
    }

    debug!(
        "Behavior: {}",
        behavior_id.clone().unwrap_or(String::from("custom"))
    );

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

    debug!("Walk Session: {ingest_session_id}");

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

            debug!("  Walk Session Path: {root_path} ({ingest_fs_path_id})");

            let rp: Vec<String> = vec![canonical_path.clone()];
            let resources =
                ResourcesCollection::from_smart_ignore(&rp, &behavior.classifier, None, false);

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
                        let inserted =
                            insert_uniform_resource(&resource, &mut urw_state, &mut urw_entry);
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
                                        error!( "[ingest_files] unable to insert UR walk session path file system entry for {} in {}: {} ({})",
                                        &inserted.uri, db_fs_path, err, INS_UR_ISFSP_ENTRY_SQL
                                        )
                                    }
                                }
                            }
                            None => {
                                error!(
                                    "[ingest_files] error extracting path info for {} in {}",
                                    canonical_path, db_fs_path
                                )
                            }
                        }
                    }
                    Err(e) => {
                        error!("[ingest_files] Error processing a resource: {}", e);
                    }
                }
            }
        }
    }
    match tx.execute(INS_UR_INGEST_SESSION_FINISH_SQL, params![ingest_session_id]) {
        Ok(_) => {}
        Err(err) => {
            error!(
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
