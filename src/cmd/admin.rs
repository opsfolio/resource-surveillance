use anyhow::Context;
use serde_rusqlite::from_rows;

use super::AdminCommands;
use super::AdminTestCommands;
use crate::ingest::ResourceMessage;
use crate::keys::key_management::{update_private_key, update_public_key};
use crate::persist::*;
use crate::resource::EncounterableResourcePathClassifier;
use crate::sign;
// use anyhow::Error;
use base64::{engine::general_purpose, Engine as _};
use indoc::indoc;
use serde_json::{json, to_string_pretty, Value};
use std::fs;
use std::fs::File;
// use std::fs::OpenOptions;
use std::io::Write;

const QUERY_UR_DS_SQL: &str = indoc! { "
    SELECT uniform_resource_id, uri, nature, size_bytes, last_modified_at, content, digital_signature 
    FROM uniform_resource
"};

pub fn verify_all_uniform_resource_signatures(db_path: &str) -> Result<(), anyhow::Error> {
    let mut log_entries: Vec<Value> = Vec::new();
    let conn = rusqlite::Connection::open(db_path)?;
    let mut stmt = conn.prepare(QUERY_UR_DS_SQL)?;

    let rows = stmt.query_map([], |row| {
        Ok((
            row.get::<_, String>("uniform_resource_id")?,
            ResourceMessage {
                uri: row.get("uri")?,
                nature: row.get("nature")?,
                size: row.get("size_bytes")?,
                content_text: row.get("content")?,
            },
            row.get::<_, String>("digital_signature")?,
        ))
    })?;

    for row_result in rows {
        let (uniform_resource_id, resource_message, digital_sig_base64): (
            String,
            ResourceMessage,
            String,
        ) = row_result?;

        let mut verification_result: String = String::new();

        let serialized_message = serde_json::to_string(&resource_message)?;

        let digital_signature = general_purpose::STANDARD
            .decode(&digital_sig_base64)
            .map_err(|e| anyhow::anyhow!("Verify: Digital signature decoding failed: {}", e))?;

        match sign::verify_signature_with_pubkey_bytes(
            serialized_message.as_bytes(),
            &digital_signature,
        ) {
            Ok(is_valid) => {
                if is_valid {
                    verification_result = "Signature verified".to_string();
                    println!("Signature verified for {}", resource_message.uri);
                } else {
                    verification_result = "Signature verification failed".to_string();
                    println!("Signature verification failed for {}", resource_message.uri);
                }
            }
            Err(err) => {
                verification_result = "Failed to verify signature".to_string();
                eprintln!(
                    "Failed to verify signature for {}: {}",
                    resource_message.uri, err
                );
            }
        };
        let log_entry = json!({
            "uniform_resource_id": uniform_resource_id,
            "verification_result": verification_result
        });
        log_entries.push(log_entry);
    }
    let json_log_array = to_string_pretty(&log_entries)?;

    let file_path = "verification_results.json";

    let mut file = File::create(file_path)?;
    file.write_all(json_log_array.as_bytes())?;

    println!("Verification results written to {}", file_path);

    Ok(())
}

// Implement methods for `AdminCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl AdminCommands {
    pub fn execute(&self, cli: &super::Cli, args: &super::AdminArgs) -> anyhow::Result<()> {
        match self {
            AdminCommands::Init {
                state_db_fs_path,
                state_db_init_sql,
                remove_existing_first,
                with_device,
                sign,
            } => {
                // TODO: For accepting a private key from the environment, you might implement an option like
                // --env-key ENV_VAR_NAME, where ENV_VAR_NAME is the name of the environment variable that contains
                // the private key. This method allows for more secure handling of keys, as they won't be directly
                // exposed in command history or logs.

                // TODO: Remember to handle these keys securely and ensure that they are encrypted or protected at all
                // stages of handling within the application. Also, proper error handling and user feedback would be
                // crucial for these features, especially since they deal with sensitive information.

                self.init(
                    cli,
                    state_db_fs_path,
                    state_db_init_sql,
                    *remove_existing_first,
                    *with_device,
                    None,
                    sign,
                )
            }
            AdminCommands::Merge {
                state_db_fs_path,
                state_db_init_sql,
                candidates,
                ignore_candidates,
                remove_existing_first,
                sql_only,
            } => self.merge(
                cli,
                state_db_fs_path,
                state_db_init_sql,
                candidates,
                ignore_candidates,
                *remove_existing_first,
                *sql_only,
            ),
            AdminCommands::Verify {
                pub_key,
                db_to_verify,
            } => self.verify(cli, pub_key.clone(), db_to_verify.clone()),
            AdminCommands::CliHelpMd => self.cli_help_markdown(),
            AdminCommands::Test(test_args) => test_args.command.execute(cli, args, test_args),
        }
    }

    fn init(
        &self,
        cli: &super::Cli,
        db_fs_path: &String,
        db_init_sql_globs: &[String],
        remove_existing_first: bool,
        with_device: bool,
        sql_script: Option<&str>,
        sign: &str,
    ) -> anyhow::Result<()> {
        if cli.debug > 0 {
            println!("Initializing {}", db_fs_path);
        }
        if !sign.is_empty() {
            let private_key_result = fs::read_to_string(sign);

            match private_key_result {
                Ok(private_key) => {
                    update_private_key(&private_key);
                }
                Err(e) => {
                    println!("Error reading private key: {}", e);
                }
            }
        }
        if remove_existing_first {
            match std::fs::remove_file(db_fs_path) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => eprintln!("[AdminCommands::init] deleting {}: {}", db_fs_path, err),
            }
        }

        let mut dbc = DbConn::new(db_fs_path, cli.debug)
            .with_context(|| format!("[AdminCommands::init] SQLite database {}", db_fs_path))?;
        let tx = dbc
            .init(Some(db_init_sql_globs))
            .with_context(|| format!("[AdminCommands::init] init transaction {}", db_fs_path))?;

        println!("Entering with_device block");
        if with_device {
            println!("In with_device block");
            let device_boundary = crate::DEVICE
                .boundary
                .as_deref()
                .unwrap_or("default_boundary");
            let message = format!("{}{}", crate::DEVICE.name, device_boundary);
            println!("invoking sign_message_with_private_keys()");
            let digital_signature = sign::sign_message_with_privkey_bytes(message.as_bytes())?;
            let digital_signature_base64 = general_purpose::STANDARD.encode(digital_signature);

            let (device_id, device_name) =
                upserted_device(&tx, &crate::DEVICE, Some(&digital_signature_base64))
                    .with_context(|| {
                        format!(
                            "[AdminCommands::init] upserted_device {} in {}",
                            crate::DEVICE.name,
                            db_fs_path
                        )
                    })?;

            if cli.debug > 0 {
                println!(
                    "Initialized {} with device {} ({})",
                    db_fs_path, device_name, device_id
                );
            }
        }

        let result = match sql_script {
            Some(sql_script) => match tx.execute_batch(sql_script) {
                Ok(_) => Ok(()),
                Err(err) => Err(err.into()),
            },
            None => Ok(()),
        };
        tx.commit()
            .with_context(|| format!("[AdminCommands::init] transaction commit {}", db_fs_path))?;
        result
    }

    fn cli_help_markdown(&self) -> anyhow::Result<()> {
        clap_markdown::print_help_markdown::<super::Cli>();
        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    fn merge(
        &self,
        cli: &super::Cli,
        state_db_fs_path: &String,
        state_db_init_sql: &[String],
        candidates: &[String],
        ignore_candidates: &[String],
        remove_existing_first: bool,
        sql_only: bool,
    ) -> Result<(), anyhow::Error> {
        let mut ignore_candidates = ignore_candidates.to_vec();
        ignore_candidates.push(state_db_fs_path.clone());

        let mut ignore_globset = globset::GlobSetBuilder::new();
        for db_ignore_path in ignore_candidates {
            match globset::GlobBuilder::new(&db_ignore_path)
                .literal_separator(true)
                .build()
            {
                Ok(glob) => {
                    let _ = ignore_globset.add(glob);
                }
                Err(err) => {
                    eprintln!(
                        "[AdminCommands::merge] invalid ignore glob {}: {}",
                        db_ignore_path, err
                    );
                    continue;
                }
            }
        }
        let ignore_globset = ignore_globset.build().unwrap();

        let mut db_paths: Vec<String> = Vec::new();
        for db_glob in candidates {
            for entry in glob::glob(db_glob).expect("Failed to read glob pattern") {
                match entry {
                    Ok(path) => {
                        if !ignore_globset.is_match(&path) {
                            db_paths.push(path.to_str().unwrap().to_owned());
                        }
                    }
                    Err(e) => println!(
                        "[AdminCommands::merge_sql] glob '{}' error {:?}",
                        db_glob, e
                    ),
                }
            }
        }

        let mut sql_script = String::from("");
        for db_path in &db_paths {
            let db_path_sql_identifier = crate::format::to_sql_friendly_identifier(db_path);
            sql_script.push_str(
                format!(
                    "ATTACH DATABASE '{}' AS {};\n",
                    db_path, db_path_sql_identifier
                )
                .as_str(),
            );
        }
        sql_script.push('\n');

        // TODO: read merge tables from CLI args or from SQLite directly, just be
        //       careful to order them properly for foreign-key contraints
        let merge_tables = &[
            "device",
            "behavior",
            "ur_ingest_session",
            "ur_ingest_session_fs_path",
            "uniform_resource",
            "uniform_resource_transform",
            "ur_ingest_session_fs_path_entry",
        ];
        for db_path in &db_paths {
            for merge_table in merge_tables {
                let db_path_sql_identifier = crate::format::to_sql_friendly_identifier(db_path);
                sql_script.push_str(
                    format!(
                        "INSERT OR IGNORE INTO {} SELECT * FROM {}.{};\n",
                        merge_table, db_path_sql_identifier, merge_table
                    )
                    .as_str(),
                );
            }
            sql_script.push('\n');
        }

        for db_path in &db_paths {
            let db_path_sql_identifier = crate::format::to_sql_friendly_identifier(db_path);
            sql_script.push_str(format!("DETACH DATABASE {};\n", db_path_sql_identifier).as_str());
        }

        if sql_only {
            print!("{}", sql_script);
            Ok(())
        } else {
            self.init(
                cli,
                state_db_fs_path,
                state_db_init_sql,
                remove_existing_first,
                false,
                Some(sql_script.as_str()),
                "", // using empty string to signal a merge.
            )
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn verify(
        &self,
        cli: &super::Cli,
        pub_key: String, // Path to the public key.
        db_to_verify: String,
    ) -> Result<(), anyhow::Error> {
        if !pub_key.is_empty() && !db_to_verify.is_empty() {
            // println!("Verifying digital signatures in DB with public key");

            let public_key_content =
                fs::read_to_string(&pub_key).context("Error reading public key file")?;
            update_public_key(&public_key_content);

            let db_path = &db_to_verify;
            verify_all_uniform_resource_signatures(db_path.as_ref())
                .context("Failed to verify signatures")?;
        } else {
            return Err(anyhow::Error::msg("Public key or DB path is empty"));
        }

        Ok(())
    }
}

impl AdminTestCommands {
    pub fn execute(
        &self,
        cli: &super::Cli,
        parent_args: &super::AdminArgs,
        cmd_args: &super::AdminTestArgs,
    ) -> anyhow::Result<()> {
        match self {
            AdminTestCommands::Classifiers {
                state_db_fs_path,
                state_db_init_sql,
                builtins,
            } => self.classifiers(
                cli,
                parent_args,
                cmd_args,
                state_db_fs_path.as_ref(),
                state_db_init_sql.as_ref(),
                *builtins,
            ),
        }
    }

    pub fn classifiers(
        &self,
        cli: &super::Cli,
        _parent_args: &super::AdminArgs,
        _cmd_args: &super::AdminTestArgs,
        state_db_fs_path: &str,
        state_db_init_sql: &[String],
        builtins: bool,
    ) -> anyhow::Result<()> {
        if builtins {
            let classifier: EncounterableResourcePathClassifier = Default::default();
            let (flaggables, rewrite) = classifier.as_formatted_tables();
            println!("{flaggables}\n");
            println!("{rewrite}\n");
            return Ok(());
        }

        let mut dbc = DbConn::new(state_db_fs_path, cli.debug)?;
        let tx = dbc.init(Some(state_db_init_sql))?;
        tx.commit()?; // in case the database was created

        let mut statement = dbc
            .conn
            .prepare("SELECT * FROM ur_ingest_resource_path_match_rule")?;
        let rows = from_rows::<crate::models_polygenix::UrIngestResourcePathMatchRule>(
            statement.query([]).unwrap(),
        );
        println!("==> `ur_ingest_resource_path_match_rule` serde rows");

        for r in rows.flatten() {
            println!("{:?}", r);
        }

        println!("==> `ur_ingest_resource_path_match_rule` rows");
        let query_result = dbc.query_result_as_formatted_table(
            r#"
            SELECT namespace as 'Name', regex as 'RE', flags as 'Flags', nature as 'Nature', description as 'Help'
              FROM ur_ingest_resource_path_match_rule"#,
            &[],
        )?;
        println!("{query_result}\n");

        println!("==> `ur_ingest_resource_path_rewrite_rule` rows");
        let query_result = dbc.query_result_as_formatted_table(
            r#"
            SELECT namespace, regex, replace, description 
              FROM ur_ingest_resource_path_rewrite_rule"#,
            &[],
        )?;
        println!("{query_result}\n");

        println!("==> What the data looks like after it's been parsed (namespace 'default')");
        match EncounterableResourcePathClassifier::default_from_conn(&dbc.conn) {
            Ok(classifier) => {
                let (flaggables, rewrite) = classifier.as_formatted_tables();
                println!("{flaggables}\n");
                println!("{rewrite}\n");
            }
            Err(err) => println!(
                "Unable to prepare EncounterableResourcePathClassifier from rules in the database:\n{:?}",
                err
            ),
        }

        Ok(())
    }
}
