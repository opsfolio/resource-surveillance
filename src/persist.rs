use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use anyhow::{Context, Result};
use comfy_table::*;
use globset::Glob;
use is_executable::IsExecutable; // adds path.is_executable
use rusqlite::functions::FunctionFlags;
use rusqlite::{types::ValueRef, Connection, Result as RusqliteResult, ToSql};
use serde_json::{json, Value as JsonValue};
use ulid::Ulid;

extern crate globwalk;

use super::device::Device;
use super::resource::*;

pub fn prepare_conn(db: &Connection) -> RusqliteResult<()> {
    declare_ulid_function(db)
}

pub fn declare_ulid_function(db: &Connection) -> RusqliteResult<()> {
    db.create_scalar_function("ulid", 0, FunctionFlags::SQLITE_UTF8, move |ctx| {
        assert_eq!(ctx.len(), 0, "called with unexpected number of arguments");
        Ok(Ulid::new().to_string())
    })
}

#[derive(Debug)]
pub struct DbConn {
    pub db_fs_path: String,
    pub conn: Connection,
    pub vebose_level: u8,
}

impl DbConn {
    // open an existing database or create a new one if it doesn't exist
    pub fn new(db_fs_path: &str, vebose_level: u8) -> Result<DbConn> {
        let db_fs_path = db_fs_path.to_string();
        let conn = Connection::open(db_fs_path.clone())
            .with_context(|| format!("[DbConn::new] SQLite database {}", db_fs_path))?;
        prepare_conn(&conn).with_context(|| {
            format!("[DbConn::new] prepare SQLite connection for {}", db_fs_path)
        })?;

        if vebose_level > 0 {
            println!("RSSD: {}", db_fs_path);
        }

        Ok(DbConn {
            db_fs_path,
            conn,
            vebose_level,
        })
    }

    // open an existing database and error out if it doesn't exist
    pub fn open(db_fs_path: &str, vebose_level: u8) -> Result<DbConn> {
        let db_fs_path = db_fs_path.to_string();
        let conn = Connection::open_with_flags(
            db_fs_path.clone(),
            rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
        )?;
        Ok(DbConn {
            db_fs_path,
            conn,
            vebose_level,
        })
    }

    pub fn init(&mut self, db_init_sql: Option<&[String]>) -> Result<rusqlite::Transaction> {
        // putting everything inside a transaction improves performance significantly
        let tx = self
            .conn
            .transaction()
            .with_context(|| format!("[DbConn::new] SQLite transaction in {}", self.db_fs_path))?;

        execute_migrations(&tx, "ingest")
            .with_context(|| format!("[DbConn::new] execute_migrations in {}", self.db_fs_path))?;

        if let Some(state_db_init_sql) = db_init_sql {
            // TODO: add the executed files into the behaviors or other activity log!?
            execute_globs_batch(
                &tx,
                &[".".to_string()],
                state_db_init_sql,
                "DbConn::new",
                self.vebose_level,
            )
            .with_context(|| {
                format!(
                    "[DbConn::new] execute_globs_batch {} in {}",
                    state_db_init_sql.join(", "),
                    self.db_fs_path
                )
            })?;
        }

        Ok(tx)
    }

    pub fn query_result_as_formatted_table(
        &self,
        query: &str,
        params: &[&dyn ToSql],
    ) -> Result<Table> {
        let mut stmt = self.conn.prepare(query)?;

        // Clone the column names to avoid borrowing issues
        let columns: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let mut table = Table::new();
        table
            .load_preset(presets::UTF8_FULL_CONDENSED)
            .apply_modifier(modifiers::UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_header(columns.clone());

        let rows = stmt.query_map(params, |row| {
            Ok((0..columns.len())
                .map(|i| match row.get_ref_unwrap(i) {
                    ValueRef::Integer(int_val) => {
                        Cell::new(int_val.to_string()).set_alignment(CellAlignment::Right)
                    }
                    ValueRef::Real(float_val) => {
                        Cell::new(float_val.to_string()).set_alignment(CellAlignment::Right)
                    }
                    ValueRef::Text(_) => Cell::new(row.get_unwrap::<usize, String>(i)),
                    _ => Cell::new(""),
                })
                .collect::<Vec<Cell>>())
        })?;

        for row in rows {
            table.add_row(row?);
        }

        Ok(table)
    }

    pub fn query_result_as_json_value(
        &self,
        query: &str,
        params: &[&dyn ToSql],
    ) -> Result<JsonValue> {
        let mut stmt = self.conn.prepare(query)?;

        // Clone the column names to avoid borrowing issues
        let columns: Vec<String> = stmt
            .column_names()
            .into_iter()
            .map(|s| s.to_string())
            .collect();

        let rows_result = stmt.query_map(params, |row| {
            let row_map: serde_json::Map<_, _> = columns
                .iter()
                .enumerate()
                .map(|(i, col_name)| match row.get_ref_unwrap(i) {
                    ValueRef::Integer(int_val) => (col_name.clone(), json!(int_val)),
                    ValueRef::Real(float_val) => (col_name.clone(), json!(float_val)),
                    ValueRef::Text(_) => match row.get::<usize, String>(i) {
                        Ok(val) => (col_name.clone(), json!(val)),
                        Err(err) => (col_name.clone(), json!(err.to_string())),
                    },
                    _ => (col_name.clone(), json!(null)),
                })
                .collect();
            Ok(JsonValue::Object(row_map))
        });

        let rows: Vec<JsonValue> = rows_result?
            .map(|row_result| row_result.unwrap_or_else(|e| json!({"error": e.to_string()})))
            .collect();

        Ok(json!(rows))
    }
}

execute_sql_batch!(bootstrap_ddl, include_str!("bootstrap.sql"));

query_sql_single!(
    select_notebook_cell_code_latest,
    "SELECT code_notebook_cell_id, interpretable_code FROM code_notebook_cell WHERE notebook_name = ?1 AND cell_name = ?2 ORDER BY created_at desc LIMIT 1",
    notebook_name: &str,
    cell_name: &str;
    code_notebook_cell_id: String,
    interpretable_code: String
);

// Executes a query to select the cells that are not in a particular state.
// Note that notebooks can have multiple versions of cells with different code
// but code_notebook_state is unique for cell_name's `from_state` to `to_state`.
query_sql_single!(
    is_notebook_cell_state,
    r"SELECT code_notebook_state_id
        FROM code_notebook_state
       WHERE code_notebook_cell_id = (SELECT code_notebook_cell_id FROM code_notebook_cell WHERE notebook_name = ?1 AND cell_name = ?2)
         AND from_state = ?3 AND to_state = ?4
       LIMIT 1",
    notebook_name: &str,
    cell_name: &str,
    from_state: &str,
    to_state: &str;
    code_notebook_cell_id: String
);

execute_sql!(
    insert_notebook_cell_state,
    r"INSERT INTO code_notebook_state (code_notebook_state_id, code_notebook_cell_id, from_state, to_state, transition_reason)
                               VALUES (ulid(), (SELECT code_notebook_cell_id FROM code_notebook_cell WHERE notebook_name = ?1 AND cell_name = ?2), ?3, ?4, ?5)",
    notebook_name: &str,
    cell_name: &str,
    from_state: &str,
    to_state: &str,
    transition_reason: &str
);

// Executes a query to select the most recently inserted cells for each all
// rows in ConstructionSqlNotebook. Code notebook cells are unique for
// notebook_name, cell_name and interpretable_code_hash which means there may
// be "duplicate" cells when interpretable_code has been edited.
query_sql_rows_no_args!(
    migratable_notebook_cells_all_with_versions,
    r#"   SELECT c.code_notebook_cell_id,
                 c.notebook_name,
                 c.cell_name,
                 c.interpretable_code,
                 c.interpretable_code_hash
           FROM code_notebook_cell c
          WHERE c.notebook_name = 'ConstructionSqlNotebook'
       ORDER BY c.cell_name"#;
    notebook_name: String,
    cell_name: String,
    interpretable_code: String,
    interpretable_code_hash: String,
    code_notebook_cell_id: String
);

// Executes a query to select the most recently inserted cells for each unique
// cell_name within the specified notebook. Code notebook cells are unique for
// notebook_name, cell_name and interpretable_code_hash which means there may
// be "duplicate" cells when interpretable_code has been edited.
query_sql_rows_no_args!(
    migratable_notebook_cells_uniq_all,
    r#"   SELECT c.code_notebook_cell_id,
                 c.notebook_name,
                 c.cell_name,
                 c.interpretable_code,
                 c.interpretable_code_hash,
                 MAX(c.created_at) AS most_recent_created_at
           FROM code_notebook_cell c
          WHERE c.notebook_name = 'ConstructionSqlNotebook'
       GROUP BY c.notebook_name, c.cell_name
       ORDER BY c.cell_name"#;
    notebook_name: String,
    cell_name: String,
    interpretable_code: String,
    interpretable_code_hash: String,
    code_notebook_cell_id: String
);

// same as migratable_notebook_cells_all, executed cells are excluded.
query_sql_rows_no_args!(
    migratable_notebook_cells_not_executed,
    r#"   SELECT c.code_notebook_cell_id,
                 c.notebook_name,
                 c.cell_name,
                 c.interpretable_code,
                 c.interpretable_code_hash,
                 MAX(c.created_at) AS most_recent_created_at
           FROM code_notebook_cell c
          WHERE c.notebook_name = 'ConstructionSqlNotebook'
            AND c.code_notebook_cell_id NOT IN (
                    SELECT s.code_notebook_cell_id
                    FROM code_notebook_state s
                    WHERE s.to_state = 'EXECUTED'
                )
       GROUP BY c.notebook_name, c.cell_name
       ORDER BY c.cell_name"#;
    notebook_name: String,
    cell_name: String,
    interpretable_code: String,
    interpretable_code_hash: String,
    code_notebook_cell_id: String
);

query_sql_rows_no_args!(
    notebook_cells_versions,
    r"  SELECT notebook_name,
               notebook_kernel_id,
               cell_name,
               COUNT(*) OVER(PARTITION BY notebook_name, cell_name) AS versions,
               code_notebook_cell_id
          FROM code_notebook_cell
      ORDER BY notebook_name, cell_name";
    notebook_kernel_id: String,
    notebook_name: String,
    cell_name: String,
    versions: usize,
    code_notebook_cell_id: String
);

query_sql_rows_no_args!(
    notebook_cell_states,
    r"SELECT cns.code_notebook_state_id,
             cnc.notebook_name,
             cnc.code_notebook_cell_id,
             cnc.cell_name,
             cnc.notebook_kernel_id,
             cns.from_state,
             cns.to_state,
             cns.transition_reason,
             cns.transitioned_at
        FROM code_notebook_state cns
        JOIN code_notebook_cell cnc ON cns.code_notebook_cell_id = cnc.code_notebook_cell_id";
    code_notebook_state_id: String,
    notebook_name: String,
    code_notebook_cell_id: String,
    cell_name: String,
    notebook_kernel_id: String,
    from_state: String,
    to_state: String,
    transition_reason: String,
    transitioned_at: String
);

// ulid() is not built into SQLite, be sure to register it with prepare_conn
query_sql_single!(
    upsert_device,
    r"INSERT INTO device (device_id, name, boundary, state, state_sysinfo) VALUES (ulid(), ?, ?, ?, ?)
      ON CONFLICT(name, state, boundary) DO UPDATE SET updated_at = CURRENT_TIMESTAMP
      RETURNING device_id, name",
    name: &str,
    boundary: &str,
    state: &str,
    state_sysinfo: &str;
    device_id: String,
    name: String
);

/// Executes a query to select notebook and cell information from the `code_notebook_cell` table.
/// The query is built dynamically based on the provided notebook and cell names.
/// It uses `LIKE` for pattern matching when a '%' is present in the filter text, otherwise it uses exact matching.
/// If no notebooks or cells are passed in, returns a list of all cells in all notebooks.
///
/// # Arguments
///
/// * `conn` - A reference to a `rusqlite::Connection`.
/// * `notebooks` - A reference to a vector of strings representing notebook names.
/// * `cells` - A reference to a vector of strings representing cell names.
///
/// # Returns
///
/// A `rusqlite::Result` containing a vector of tuples, each containing:
/// - `notebook_name`: The name of the notebook.
/// - `notebook_kernel_id`: The kernel ID associated with the notebook.
/// - `cell_name`: The name of the cell.
/// - `interpretable_code`: The code content of the cell.
///
/// # Examples
///
/// ```
/// # use rusqlite::{Connection, Result as SqliteResult};
/// # use std::vec::Vec;
/// # fn main() -> SqliteResult<()> {
/// let conn = Connection::open("code_notebooks.db")?;
/// prepare_conn(&conn)?; // make sure to register custom functions like ulid()
/// let notebooks = vec!["Notebook1".to_string(), "Notebook2".to_string()];
/// let cells = vec!["CellA".to_string(), "CellB".to_string()];
/// let results = select_notebooks_and_cells(&conn, &notebooks, &cells)?;
/// for (notebook_name, notebook_kernel_id, cell_name, interpretable_code) in results {
///     println!("Notebook: {}, Kernel ID: {}, Cell: {}, Code: {}", notebook_name, notebook_kernel_id, cell_name, interpretable_code);
/// }
/// # Ok(())
/// # }
/// ```
pub fn select_notebooks_and_cells(
    conn: &Connection,
    notebooks: &Vec<String>,
    cells: &Vec<String>,
) -> RusqliteResult<Vec<(String, String, String, String)>> {
    let mut query = String::from(
        "SELECT notebook_name, notebook_kernel_id, cell_name, interpretable_code \
         FROM code_notebook_cell WHERE",
    );

    let mut conditions = Vec::new();

    // Helper closure to determine whether to use LIKE or =
    let condition = |field: &str, value: &String| {
        if value.contains('%') {
            format!(" {} LIKE '{}'", field, value.replace('\'', "''")) // Escape single quotes
        } else {
            format!(" {} = '{}'", field, value.replace('\'', "''")) // Escape single quotes
        }
    };

    // Add conditions for notebook and cell combinations
    for notebook in notebooks {
        let notebook_condition = condition("notebook_name", notebook);

        if cells.is_empty() {
            // If there are no cells, add condition for notebook only
            conditions.push(notebook_condition);
        } else {
            for cell in cells {
                // Add condition for the combination of notebook and cell
                let cell_condition = condition("cell_name", cell);
                conditions.push(format!("({} AND {})", notebook_condition, cell_condition));
            }
        }
    }

    // Add individual OR conditions for cells if cells are provided
    if !cells.is_empty() {
        for cell in cells {
            let cell_condition = condition("cell_name", cell);
            conditions.push(cell_condition);
        }
    }

    // In case no notebooks or cells combinations are passed, assume all cells
    if conditions.is_empty() {
        conditions.push(String::from(" cell_name LIKE '%'"))
    }

    // Join all conditions with OR
    query.push_str(&conditions.join(" OR "));
    query.push_str("ORDER BY notebook_name, cell_name");

    let mut statement = conn.prepare(&query)?;
    let notebook_cell_iter = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            row.get::<_, String>(1)?,
            row.get::<_, String>(2)?,
            row.get::<_, String>(3)?,
        ))
    })?;

    // Collect the results into a vector
    let results: Result<Vec<_>, _> = notebook_cell_iter.collect();
    results
}

/**
 * IMPORTANT TODO: ensure all high performance loops are wrapped in prepare
 * statements along with BEGIN/END transactions in batches (prepared stmts
 * are faster than non-prepared, prepared stmts inside transactions are
 * faster than those outside of transactions).
 * See: https://tedspence.com/investigating-rust-with-sqlite-53d1f9a41112
 *
 * - TODO: Turn on foreign key and other pragmas.
 * - Try to make sure every SQL statement is idempotent (using ON CONFLICT).
 * - Use direct Rusqlite when raw SQLite access is important for performance,
 *   SQLx when mid-level access is necessary and SQL might be used across
 *   multiple databases, SeaORM when performace is less important than
 *   convenience and type-safety.
 */

pub enum ExecutableCode {
    NotebookCell {
        notebook_name: String,
        cell_name: String,
    },
    _AnonymousSql {
        sql: String,
    },
    _Sql {
        identifier: String,
        sql: String,
    },
}

impl ExecutableCode {
    pub fn executable_code_latest(&self, conn: &Connection) -> RusqliteResult<String> {
        match self {
            ExecutableCode::NotebookCell {
                notebook_name,
                cell_name,
            } => match select_notebook_cell_code_latest(conn, notebook_name, cell_name) {
                Ok((_id, code)) => Ok(code),
                Err(err) => Err(err),
            },
            ExecutableCode::_AnonymousSql { sql } | ExecutableCode::_Sql { sql, .. } => {
                Ok(sql.clone())
            }
        }
    }

    pub fn _hash_key(&self) -> String {
        match self {
            ExecutableCode::_Sql { identifier, .. } => identifier.clone(),
            ExecutableCode::_AnonymousSql { sql } => {
                let mut hasher = DefaultHasher::new();
                sql.hash(&mut hasher);
                format!("{:x}", hasher.finish())
            }
            ExecutableCode::NotebookCell {
                notebook_name,
                cell_name,
            } => {
                format!("{}::{}", notebook_name, cell_name)
            }
        }
    }
}

pub fn execute_migrations(conn: &Connection, context: &str) -> RusqliteResult<()> {
    // bootstrap_ddl is idempotent and should be called at start of every session
    // because it contains notebook entries, SQL used in migrations, etc.
    let _ = bootstrap_ddl(conn);
    migratable_notebook_cells_uniq_all(
        conn,
        |_index, notebook_name, cell_name, sql, _hash, id: String| {
            if cell_name.contains("_once_") {
                match execute_batch_stateful(
                    conn,
                    &ExecutableCode::NotebookCell {
                        notebook_name: notebook_name.clone(),
                        cell_name: cell_name.clone(),
                    },
                    "NONE",
                    "EXECUTED",
                    "execute_migrations",
                ) {
                    None => {
                        println!(
                            "[TODO: move this to Otel, {}] {} {} migration not required ({})",
                            context, notebook_name, cell_name, id
                        );
                        Ok(())
                    }
                    Some(_) => {
                        println!(
                            "[TODO: move this to Otel, {}] {} {} migrated ({})",
                            context, notebook_name, cell_name, id
                        );
                        Ok(())
                    }
                }
            } else {
                println!(
                    "[TODO: move this to Otel, {}] {} {} migrated ({})",
                    context, notebook_name, cell_name, id
                );
                conn.execute_batch(&sql)
            }
        },
    )
}

pub fn execute_globs_batch(
    conn: &Connection,
    walk_paths: &[String],
    candidates_globs: &[String],
    context: &str,
    verbose_level: u8,
) -> anyhow::Result<Vec<(String, Option<String>, bool)>> {
    let mut executed: Vec<(String, Option<String>, bool)> = Vec::new();

    // prepare a single walker which will build a GlobWalker for each walk_path,
    // and iterate through only valid DirEntries;
    // TODO: this "eats" all errors without reporting
    let entries = walk_paths
        .iter()
        .map(|bp: &String| {
            globwalk::GlobWalkerBuilder::from_patterns(bp, candidates_globs)
                .follow_links(true)
                .build()
        })
        .filter_map(Result::ok)
        .flatten()
        .filter_map(Result::ok);

    let capturables_glob = Glob::new("*.sql.{ts,sh}")?.compile_matcher();
    for entry in entries {
        let path = entry.path();
        let uri = path.to_string_lossy().to_string();
        let (sql, is_captured_from_exec) =
            if capturables_glob.is_match(path) && path.is_executable() {
                let command = path.to_string_lossy().to_string();
                let ce = CapturableExecutable::UriShellExecutive(
                    Box::new(command.clone()), // `String` has ShellExecutive trait
                    command,
                    String::from("surveilr-SQL"), // arbitrary but useful "nature"
                    true,
                );
                match ce.executed_result_as_sql(crate::shell::ShellStdIn::None) {
                    Ok((sql_from_captured_exec, _nature)) => (sql_from_captured_exec, true),
                    Err(err) => {
                        eprintln!(
                            "[execute_globs_batch({})] Unable to execute {}:\n{}",
                            context, uri, err
                        );
                        continue;
                    }
                }
            } else {
                match std::fs::read_to_string(path) {
                    Ok(sql_from_file) => (sql_from_file, false),
                    Err(err) => {
                        eprintln!(
                            "[execute_globs_batch({})] Failed to read SQL file {}: {}",
                            context, uri, err
                        );
                        continue;
                    }
                }
            };

        match conn.execute_batch(&sql) {
            Ok(_) => {
                executed.push((
                    entry.path().to_string_lossy().to_string(),
                    Some(sql),
                    is_captured_from_exec,
                ));
            }
            Err(e) => {
                executed.push((
                    entry.path().to_string_lossy().to_string(),
                    None,
                    is_captured_from_exec,
                ));
                eprintln!(
                    "[execute_globs_batch({})] Failed to execute SQL file: {}",
                    context, e
                );
            }
        }
    }

    if verbose_level > 0 {
        let emit: Vec<String> = executed
            .iter()
            .map(|i| {
                format!(
                    "{} ({} lines{})",
                    i.0,
                    if i.1.is_some() {
                        i.1.clone().unwrap().lines().count()
                    } else {
                        0
                    },
                    if i.2 { "*" } else { "" } // * means it was a captured executable
                )
            })
            .collect();
        if !emit.is_empty() {
            println!(
                "[{}] executed SQL batches from: {}",
                context,
                emit.join(", ")
            )
        } else {
            println!(
                "[{}] did execute SQL batches, none requested/matched {}",
                context,
                candidates_globs.join(", ")
            )
        }
    }

    Ok(executed)
}

pub fn execute_batch(conn: &Connection, ec: &ExecutableCode) -> RusqliteResult<()> {
    match ec.executable_code_latest(conn) {
        Ok(sql) => conn.execute_batch(&sql),
        Err(err) => Err(err),
    }
}

pub fn execute_batch_stateful(
    conn: &Connection,
    ec: &ExecutableCode,
    from_state: &str,
    to_state: &str,
    transition_reason: &str,
) -> Option<RusqliteResult<()>> {
    match ec {
        ExecutableCode::NotebookCell {
            notebook_name,
            cell_name,
        } => match is_notebook_cell_state(conn, notebook_name, cell_name, from_state, to_state) {
            Ok(_) => None,
            Err(rusqlite::Error::QueryReturnedNoRows) => match execute_batch(conn, ec) {
                Ok(_) => {
                    match insert_notebook_cell_state(
                        conn,
                        notebook_name,
                        cell_name,
                        from_state,
                        to_state,
                        transition_reason,
                    ) {
                        Ok(_) => Some(Ok(())),
                        Err(err) => Some(Err(err)),
                    }
                }
                Err(err) => Some(Err(err)),
            },
            Err(err) => Some(Err(err)),
        },
        // TODO: instead of always executing, insert the SQL to code_notebook_cell
        //       in a notebook called `execute_batch_stateful` with ec.hash_key as
        //       the cell name, then recursively call execute_batch_stateful with
        //       that new cell; this way we can track it and only run once
        _ => Some(execute_batch(conn, ec)),
    }
}

pub fn upserted_device(conn: &Connection, device: &Device) -> RusqliteResult<(String, String)> {
    upsert_device(
        conn,
        &device.name,
        if let Some(boundary) = &device.boundary {
            boundary
        } else {
            "UNKNOWN"
        },
        &device.state_json(),
        &device.state_sysinfo_json(),
    )
}
