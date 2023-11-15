use anyhow::Context;
use rusqlite::Connection;

use super::AdminCommands;
use crate::persist::*;

// Implement methods for `NotebooksCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl AdminCommands {
    pub fn execute(&self, cli: &super::Cli, _args: &super::AdminArgs) -> anyhow::Result<()> {
        match self {
            AdminCommands::Init {
                state_db_fs_path,
                remove_existing_first,
            } => self.init(cli, state_db_fs_path, *remove_existing_first),
            AdminCommands::MergeSql {
                db_glob,
                db_glob_ignore,
            } => self.merge_sql(db_glob, db_glob_ignore),
            AdminCommands::CliHelpMd => self.cli_help_markdown(),
        }
    }

    fn init(
        &self,
        cli: &super::Cli,
        db_fs_path: &String,
        remove_existing_first: bool,
    ) -> anyhow::Result<()> {
        if cli.debug == 1 {
            println!("Initializing {}", db_fs_path);
        }
        if remove_existing_first {
            match std::fs::remove_file(db_fs_path) {
                Ok(_) => {}
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
                Err(err) => eprintln!("[AdminCommands::init] deleting {}: {}", db_fs_path, err),
            }
        }

        let conn = Connection::open(db_fs_path)
            .with_context(|| format!("[AdminCommands::init] SQLite database {}", db_fs_path))?;

        // add all our custom functions (`ulid()`, etc.)
        prepare_conn(&conn).with_context(|| {
            format!(
                "[AdminCommands::init] prepare SQLite connection for {}",
                db_fs_path
            )
        })?;

        execute_migrations(&conn, "AdminCommands::init").with_context(|| {
            format!("[AdminCommands::init] execute_migrations in {}", db_fs_path)
        })?;

        // insert the device or, if it exists, get its current ID and name
        let (device_id, device_name) =
            upserted_device(&conn, &crate::DEVICE).with_context(|| {
                format!(
                    "[AdminCommands::init] upserted_device {} in {}",
                    crate::DEVICE.name,
                    db_fs_path
                )
            })?;

        if cli.debug == 1 {
            println!(
                "Initialized {} with device {} ({})",
                db_fs_path, device_name, device_id
            );
        }

        Ok(())
    }

    fn cli_help_markdown(&self) -> anyhow::Result<()> {
        clap_markdown::print_help_markdown::<super::Cli>();
        Ok(())
    }

    fn merge_sql(
        &self,
        db_globs: &[String],
        db_ignore_globs: &[String],
    ) -> Result<(), anyhow::Error> {
        let mut ignore_globset = globset::GlobSetBuilder::new();
        for db_ignore_path in db_ignore_globs {
            match globset::GlobBuilder::new(db_ignore_path)
                .literal_separator(true)
                .build()
            {
                Ok(glob) => {
                    let _ = ignore_globset.add(glob);
                }
                Err(err) => {
                    eprintln!(
                        "[AdminCommands::merge_sql] invalid ignore glob {}: {}",
                        db_ignore_path, err
                    );
                    continue;
                }
            }
        }
        let ignore_globset = ignore_globset.build().unwrap();

        let mut db_paths: Vec<String> = Vec::new();
        for db_glob in db_globs {
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

        let merge_tables = &[
            "device",
            "ur_walk_session",
            "ur_walk_session_path",
            "uniform_resource",
            "ur_walk_session_path_fs_entry",
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
        print!("{}", sql_script);
        Ok(())
    }
}
