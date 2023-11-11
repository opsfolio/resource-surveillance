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
                surveil_db_fs_path,
                remove_existing_first,
            } => self.init(cli, surveil_db_fs_path, *remove_existing_first),
            AdminCommands::CliHelpMd => self.ls(),
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
            std::fs::remove_file(db_fs_path)
                .with_context(|| format!("[AdminCommands::init] deleting {}", db_fs_path))?;
            if cli.debug == 1 {
                println!("Removed {} by request", db_fs_path);
            }
        }

        let conn = Connection::open(db_fs_path)
            .with_context(|| format!("[AdminCommands::init] SQLite database {}", db_fs_path))?;

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

    fn ls(&self) -> anyhow::Result<()> {
        clap_markdown::print_help_markdown::<super::Cli>();
        Ok(())
    }
}
