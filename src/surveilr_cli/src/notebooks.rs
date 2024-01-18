use autometrics::autometrics;
use rusqlite::{Connection, OpenFlags};
use tracing::error;
use tracing::info;

use cmd::{NotebooksCommands, NotebooksArgs};
use common::format::*;
use resource_serde::persist::*;

// Implement methods for `NotebooksCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
#[derive(Debug, Default)]
pub struct Notebooks {}

impl Notebooks {
    #[autometrics]
    pub fn execute(&self, _cli: &super::Cli, args: &NotebooksArgs) -> anyhow::Result<()> {
        match &args.command {
            NotebooksCommands::Cat {
                notebook,
                cell,
                seps,
            } => self.cat(args, notebook, cell, *seps),
            NotebooksCommands::Ls { migratable } => {
                if *migratable {
                    self.ls_migrations(args)
                } else {
                    self.ls(args)
                }
            }
        }
    }

    fn cat(
        &self,
        args: &NotebooksArgs,
        notebooks: &Vec<String>,
        cells: &Vec<String>,
        seps: bool,
    ) -> anyhow::Result<()> {
        if let Some(db_fs_path) = args.state_db_fs_path.as_deref() {
            if let Ok(conn) =
                Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            {
                match select_notebooks_and_cells(&conn, notebooks, cells) {
                    Ok(matched) => {
                        for row in matched {
                            let (notebook, kernel, cell, code) = row;
                            if seps {
                                println!("-- {notebook}::{cell} ({kernel})");
                            }
                            println!("{code}");
                        }
                    }
                    Err(err) => error!("Notebooks cells command error: {}", err),
                }
            } else {
                error!(
                    "Notebooks cells command requires a database: {}",
                    db_fs_path
                );
            }
        }
        Ok(())
    }

    fn ls(&self, args: &NotebooksArgs) -> anyhow::Result<()> {
        if let Some(db_fs_path) = args.state_db_fs_path.as_deref() {
            if let Ok(conn) =
                Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            {
                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                notebook_cells_versions(&conn, |_index, kernel, nb, cell: String, versions, id| {
                    rows.push(vec![nb, kernel, cell, versions.to_string(), id]);
                    Ok(())
                })
                .unwrap();
                println!(
                    "{}",
                    as_ascii_table(&["Notebook", "Kernel", "Cell", "Versions", "ID"], &rows)
                );
            } else {
                info!("Notebooks command requires a database: {}", db_fs_path);
            };
        }
        Ok(())
    }

    fn ls_migrations(&self, args: &NotebooksArgs) -> anyhow::Result<()> {
        if let Some(db_fs_path) = args.state_db_fs_path.as_deref() {
            if let Ok(conn) =
                Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            {
                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                migratable_notebook_cells_all_with_versions(
                    &conn,
                    |_index, notebook_name, cell_name, _sql, hash, id: String| {
                        rows.push(vec![notebook_name, cell_name, hash, id]);
                        Ok(())
                    },
                )
                .unwrap();
                info!("All cells that are candidates for migration (including duplicates)");
                println!(
                    "{}",
                    as_ascii_table(&["Notebook", "Cell", "Code Hash", "ID"], &rows)
                );

                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                migratable_notebook_cells_uniq_all(
                    &conn,
                    |_index, notebook_name, cell_name, _sql, hash, id: String| {
                        rows.push(vec![notebook_name, cell_name, hash, id]);
                        Ok(())
                    },
                )
                .unwrap();
                info!("All cells deemed to be migratable (unique rows)");
                println!(
                    "{}",
                    as_ascii_table(&["Notebook", "Cell", "Code Hash", "ID"], &rows)
                );

                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                migratable_notebook_cells_not_executed(
                    &conn,
                    |_index, notebook_name, cell_name, _sql, hash, id: String| {
                        rows.push(vec![notebook_name, cell_name, hash, id]);
                        Ok(())
                    },
                )
                .unwrap();
                info!("All cells that should be migrated because they have not been executed");
                println!(
                    "{}",
                    as_ascii_table(&["Notebook", "Cell", "Code Hash", "ID"], &rows)
                );

                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                notebook_cell_states(
                    &conn,
                    |_index,
                     _code_notebook_state_id,
                     notebook_name,
                     code_notebook_cell_id: String,
                     cell_name,
                     notebook_kernel_id,
                     from_state,
                     to_state,
                     transition_reason,
                     transitioned_at| {
                        rows.push(vec![
                            notebook_name,
                            notebook_kernel_id,
                            cell_name,
                            from_state,
                            to_state,
                            transition_reason,
                            transitioned_at,
                            code_notebook_cell_id,
                        ]);
                        Ok(())
                    },
                )
                .unwrap();
                info!("code_notebook_state");
                println!(
                    "{}",
                    as_ascii_table(
                        &[
                            "Notebook", "Kernel", "Cell", "From", "To", "Remarks", "When",
                            "Cell ID"
                        ],
                        &rows
                    )
                );
            } else {
                println!("Notebooks command requires a database: {}", db_fs_path);
            };
        }
        Ok(())
    }
}
