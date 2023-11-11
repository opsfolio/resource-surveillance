use anyhow::{Context, Result};
use indoc::indoc;
use rusqlite::{params, Connection};

use crate::fsresource::*;
use crate::persist::*;
use crate::resource::*;

pub fn fs_walk(cli: &super::Cli, args: &super::FsWalkArgs) -> Result<String> {
    let db_fs_path = &args.surveil_db_fs_path;

    if cli.debug == 1 {
        println!("Surveillance DB: {}", db_fs_path);
    }

    let mut conn = Connection::open(db_fs_path)
        .with_context(|| format!("[fs_walk] SQLite database {}", db_fs_path))?;

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

    let walk_session_id = ulid::Ulid::new().to_string();
    if cli.debug == 1 {
        println!("Walk Session: {walk_session_id}");
    }

    // separate the SQL from the execute so we can use it in logging, errors, etc.
    const INS_UR_WALK_SESSION_SQL: &str = indoc! {"
        INSERT INTO ur_walk_session (ur_walk_session_id, device_id, ignore_paths_regex, blobs_regex, digests_regex, walk_started_at) 
                             VALUES (?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"};
    const INS_UR_WALK_SESSION_FINISH_SQL: &str = indoc! {"
        UPDATE ur_walk_session 
           SET walk_finished_at = CURRENT_TIMESTAMP 
         WHERE ur_walk_session_id = ?"};
    const INS_UR_WSP_SQL: &str = indoc! {"
        INSERT INTO ur_walk_session_path (ur_walk_session_path_id, walk_session_id, root_path) 
                                  VALUES (?, ?, ?)"};
    // in ins_ur_stmt the `DO UPDATE SET size_bytes = EXCLUDED.size_bytes` is a workaround to force uniform_resource_id when the row already exists
    const INS_UR_SQL: &str = indoc! {"
        INSERT INTO uniform_resource (uniform_resource_id, device_id, walk_session_id, walk_path_id, uri, nature, content, content_digest, size_bytes, last_modified_at, content_fm_body_attrs, frontmatter)
                              VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) 
                         ON CONFLICT (device_id, content_digest, uri, size_bytes, last_modified_at) 
                           DO UPDATE SET size_bytes = EXCLUDED.size_bytes
                           RETURNING uniform_resource_id"};
    const INS_UR_FS_ENTRY_SQL: &str = indoc! {"
        INSERT INTO ur_walk_session_path_fs_entry (ur_walk_session_path_fs_entry_id, walk_session_id, walk_path_id, uniform_resource_id, file_path_abs, file_path_rel_parent, file_path_rel, file_basename, file_extn) 
                                           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)"};

    tx.execute(
        INS_UR_WALK_SESSION_SQL,
        params![
            walk_session_id,
            device_id,
            args.ignore_entry
                .iter()
                .map(|r| r.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
            args.compute_digests
                .iter()
                .map(|r| r.as_str())
                .collect::<Vec<&str>>()
                .join(", "),
            args.surveil_content
                .iter()
                .map(|r| r.as_str())
                .collect::<Vec<&str>>()
                .join(", ")
        ],
    )
    .with_context(|| {
        format!(
            "[fs_walk] inserting UR walk session using {} in {}",
            INS_UR_WALK_SESSION_SQL, db_fs_path
        )
    })?;

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

        for root_path in &args.root_path {
            let canonical_path_buf = std::fs::canonicalize(std::path::Path::new(&root_path))
                .with_context(|| {
                    format!(
                        "[fs_walk] unable to canonicalize {} in {}",
                        root_path, db_fs_path
                    )
                })?;
            let canonical_path = canonical_path_buf.into_os_string().into_string().unwrap();
            let walk_path_id = ulid::Ulid::new().to_string();
            if cli.debug == 1 {
                println!("  Walk Session Path: {root_path} ({walk_path_id})");
            }

            let ins_ur_wsp_params = params![walk_path_id, walk_session_id, canonical_path];
            ins_ur_wsp_stmt
                .execute(ins_ur_wsp_params)
                .with_context(|| {
                    format!(
                        "[fs_walk] ins_ur_wsp_stmt {} with {} in {}",
                        INS_UR_WSP_SQL, "TODO: ins_ur_wsp_params.join()", db_fs_path
                    )
                })?;

            let rp: Vec<String> = vec![canonical_path.clone()];
            let walker =
                FileSysResourcesWalker::new(&rp, &args.ignore_entry, &args.surveil_content)
                    .with_context(|| {
                        format!(
                            "[fs_walk] unable to walker for {} in {}",
                            canonical_path, db_fs_path
                        )
                    })?;

            for resource_result in walker.walk_resources_iter() {
                match resource_result {
                    Ok(resource) => {
                        // compute a new Ulid in case case a new row is added, it will be
                        // replaced by the value of the RETURNING clause, though, in case
                        // the content is already in the database.
                        let mut uniform_resource_id = ulid::Ulid::new().to_string();
                        let uri: String;
                        match resource {
                            UniformResource::Html(html) => {
                                uri = html.resource.uri.to_string();
                                // TODO: this will panic if content not available, so test for proper existence not unwrap()!
                                let content_supplier =
                                    html.resource.content_text_supplier.unwrap()().unwrap();
                                match ins_ur_stmt.query_row(
                                    params![
                                        uniform_resource_id,
                                        device_id,
                                        walk_session_id,
                                        walk_path_id,
                                        html.resource.uri,
                                        html.resource.nature,
                                        content_supplier.content_text(),
                                        content_supplier.content_digest_hash(),
                                        html.resource.size,
                                        html.resource.last_modified_at.unwrap().to_string(),
                                        &None::<String>,
                                        &None::<String>
                                    ],
                                    |row| row.get(0),
                                ) {
                                    Ok(existing_ur_id) => uniform_resource_id = existing_ur_id, // this happens with RETURNING for existing row
                                    Err(rusqlite::Error::QueryReturnedNoRows) => {} // this happens with a new row
                                    Err(err) => {
                                        eprintln!(
                                            "Error inserting UniformResource::Html for {}: {:?}",
                                            &uri, err
                                        );
                                    }
                                }
                                // TODO: parse HTML and store HTML <head><meta> as frontmatter
                            }
                            UniformResource::Json(json) => {
                                uri = json.resource.uri.to_string();
                                // TODO: this will panic if content not available, so test for proper existence not unwrap()!
                                let content_supplier =
                                    json.resource.content_text_supplier.unwrap()().unwrap();
                                match ins_ur_stmt.query_row(
                                    params![
                                        uniform_resource_id,
                                        device_id,
                                        walk_session_id,
                                        walk_path_id,
                                        json.resource.uri,
                                        json.resource.nature,
                                        content_supplier.content_text(),
                                        content_supplier.content_digest_hash(),
                                        json.resource.size,
                                        json.resource.last_modified_at.unwrap().to_string(),
                                        &None::<String>,
                                        &None::<String>
                                    ],
                                    |row| row.get(0),
                                ) {
                                    Ok(existing_ur_id) => uniform_resource_id = existing_ur_id, // this happens with RETURNING for existing row
                                    Err(rusqlite::Error::QueryReturnedNoRows) => {} // this happens with a new row
                                    Err(err) => {
                                        eprintln!(
                                            "Error inserting UniformResource::Json for {}: {:?}",
                                            &uri, err
                                        );
                                    }
                                }
                            }
                            UniformResource::Image(img) => {
                                uri = img.resource.uri.to_string();
                                let mut digest_hash: String = String::from("-");
                                if let Some(binary_supplier) = img.resource.content_binary_supplier
                                {
                                    if let Ok(binary_supplier) = binary_supplier() {
                                        digest_hash =
                                            binary_supplier.content_digest_hash().to_string();
                                    }
                                }
                                match ins_ur_stmt.query_row(
                                    params![
                                        uniform_resource_id,
                                        device_id,
                                        walk_session_id,
                                        walk_path_id,
                                        img.resource.uri,
                                        img.resource.nature,
                                        &None::<String>, // TODO: should we store the binaries?
                                        digest_hash,
                                        img.resource.size,
                                        img.resource.last_modified_at.unwrap().to_string(),
                                        &None::<String>,
                                        &None::<String>
                                    ],
                                    |row| row.get(0),
                                ) {
                                    Ok(existing_ur_id) => uniform_resource_id = existing_ur_id, // this happens with RETURNING for existing row
                                    Err(rusqlite::Error::QueryReturnedNoRows) => {} // this happens with a new row
                                    Err(err) => {
                                        eprintln!(
                                            "Error inserting UniformResource::Image for {}: {:?}",
                                            &uri, err
                                        );
                                    }
                                }
                            }
                            UniformResource::Markdown(md) => {
                                uri = md.resource.uri.to_string();
                                // TODO: this will panic if content not available, so test for proper existence not unwrap()!
                                let content_supplier =
                                    md.resource.content_text_supplier.unwrap()().unwrap();
                                let mut fm_attrs = None::<String>;
                                let mut fm_json: Option<String> = None::<String>;
                                let (_, fm_raw, fm_json_value, fm_body) =
                                    content_supplier.frontmatter();
                                if fm_json_value.is_ok() {
                                    fm_json = Some(
                                        serde_json::to_string_pretty(&fm_json_value.ok()).unwrap(),
                                    );
                                    let fm_attrs_value = serde_json::json!({
                                        "frontMatter": fm_raw.unwrap(),
                                        "body": fm_body,
                                        "attrs": fm_json.clone().unwrap()
                                    });
                                    fm_attrs = Some(
                                        serde_json::to_string_pretty(&fm_attrs_value).unwrap(),
                                    );
                                }
                                match ins_ur_stmt.query_row(
                                    params![
                                        uniform_resource_id,
                                        device_id,
                                        walk_session_id,
                                        walk_path_id,
                                        md.resource.uri,
                                        md.resource.nature,
                                        content_supplier.content_text(),
                                        content_supplier.content_digest_hash(),
                                        md.resource.size,
                                        md.resource.last_modified_at.unwrap().to_string(),
                                        fm_attrs,
                                        fm_json
                                    ],
                                    |row| row.get(0),
                                ) {
                                    Ok(existing_ur_id) => uniform_resource_id = existing_ur_id, // this happens with RETURNING for existing row
                                    Err(rusqlite::Error::QueryReturnedNoRows) => {} // this happens with a new row
                                    Err(err) => {
                                        eprintln!(
                                        "Error inserting UniformResource::Markdown for {}: {:?}",
                                        &uri, err
                                    );
                                    }
                                }
                            }
                            UniformResource::Unknown(unknown) => {
                                uri = unknown.uri.to_string();
                                match ins_ur_stmt.query_row(
                                    params![
                                        uniform_resource_id,
                                        device_id,
                                        walk_session_id,
                                        walk_path_id,
                                        unknown.uri,
                                        unknown.nature,
                                        &None::<String>,
                                        String::from("-"),
                                        unknown.size,
                                        unknown.last_modified_at.unwrap().to_string(),
                                        &None::<String>,
                                        &None::<String>
                                    ],
                                    |row| row.get(0),
                                ) {
                                    Ok(existing_ur_id) => uniform_resource_id = existing_ur_id, // this happens with RETURNING for existing row
                                    Err(rusqlite::Error::QueryReturnedNoRows) => {} // this happens with a new row
                                    Err(err) => {
                                        eprintln!(
                                            "Error inserting UniformResource::Unknown for {}: {:?}",
                                            &uri, err
                                        );
                                    }
                                }
                            }
                        }

                        match extract_path_info(
                            std::path::Path::new(&canonical_path),
                            std::path::Path::new(&uri),
                        ) {
                            Some((
                                file_path_abs,
                                file_path_rel_parent,
                                file_path_rel,
                                file_basename,
                                file_extn,
                            )) => {
                                let ur_walk_session_path_fs_entry_id =
                                    ulid::Ulid::new().to_string();

                                match ins_ur_fs_entry_stmt.execute(params![
                                    ur_walk_session_path_fs_entry_id,
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
                                    }
                                ]) {
                                    Ok(_) => {}
                                    Err(err) => {
                                        eprintln!( "[fs_walk] unable to insert UR walk session path file system entry for {} in {}: {} ({})",
                                            &uri, db_fs_path, err, INS_UR_FS_ENTRY_SQL
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
