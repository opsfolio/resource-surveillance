use anyhow::Context;
use serde_rusqlite::from_rows;

use super::AdminCommands;
use super::AdminTestCommands;
use crate::persist::*;
use crate::resource::EncounterableResourcePathClassifier;

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
            } => self.init(
                cli,
                state_db_fs_path,
                state_db_init_sql,
                *remove_existing_first,
                *with_device,
                None,
            ),
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
    ) -> anyhow::Result<()> {
        if cli.debug > 0 {
            println!("Initializing {}", db_fs_path);
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

        if with_device {
            // insert the device or, if it exists, get its current ID and name
            let (device_id, device_name) =
                upserted_device(&tx, &crate::DEVICE).with_context(|| {
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
            )
        }
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
