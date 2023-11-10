use rusqlite::{Connection, OpenFlags};

use super::NotebooksCommands;
use crate::format::*;
use crate::persist::*;

// Implement methods for `NotebooksCommands`, ensure that whether the commands
// are called from CLI or natively within Rust, all the calls remain ergonomic.
impl NotebooksCommands {
    pub fn execute(&self, _cli: &super::Cli, args: &super::NotebooksArgs) -> anyhow::Result<()> {
        match self {
            NotebooksCommands::Cat {
                notebook,
                cell,
                seps,
            } => self.cat(args, notebook, cell, *seps),
            NotebooksCommands::Ls => self.ls(args),
        }
    }

    fn cat(
        &self,
        args: &super::NotebooksArgs,
        notebooks: &Vec<String>,
        cells: &Vec<String>,
        seps: bool,
    ) -> anyhow::Result<()> {
        if let Some(db_fs_path) = args.surveil_db_fs_path.as_deref() {
            if let Ok(conn) =
                Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            {
                match select_notebooks_and_cells(&conn, notebooks, cells) {
                    Ok(matched) => {
                        for row in matched {
                            let (notebook, kernel, cell, sql) = row;
                            if seps {
                                println!("-- {notebook}::{cell} ({kernel})");
                            }
                            println!("{sql}");
                        }
                    }
                    Err(err) => println!("Notebooks cells command error: {}", err),
                }
            } else {
                println!(
                    "Notebooks cells command requires a database: {}",
                    db_fs_path
                );
            }
        }
        Ok(())
    }

    fn ls(&self, args: &super::NotebooksArgs) -> anyhow::Result<()> {
        if let Some(db_fs_path) = args.surveil_db_fs_path.as_deref() {
            if let Ok(conn) =
                Connection::open_with_flags(db_fs_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            {
                let mut rows: Vec<Vec<String>> = Vec::new(); // Declare the rows as a vector of vectors of strings
                notebook_cells(&conn, |_index, kernel, nb, cell, id| {
                    rows.push(vec![nb, kernel, cell, id]);
                    Ok(())
                })
                .unwrap();
                println!(
                    "{}",
                    format_table(&["Notebook", "Kernel", "Cell", "ID"], &rows)
                );

                rows = Vec::new(); // Declare the rows as a vector of vectors of strings
                notebook_cell_states(
                    &conn,
                    |_index,
                     _code_notebook_state_id,
                     notebook_name,
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
                        ]);
                        Ok(())
                    },
                )
                .unwrap();
                println!(
                    "{}",
                    format_table(
                        &["Notebook", "Kernel", "Cell", "From", "To", "Remarks", "When"],
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
