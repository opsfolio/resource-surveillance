use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use rusqlite::{CachedStatement, Connection, Result as RusqliteResult, ToSql};
use ulid::Ulid;

use super::device::Device;

execute_sql_batch!(bootstrap_ddl, include_str!("bootstrap.sql"));

query_sql_single!(
    select_notebook_cell_code,
    "SELECT code_notebook_cell_id, interpretable_code FROM code_notebook_cell WHERE notebook_name = ?1 AND cell_name = ?2",
    notebook_name: &str,
    cell_name: &str;
    code_notebook_cell_id: String,
    interpretable_code: String
);

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
                               VALUES (?3, (SELECT code_notebook_cell_id FROM code_notebook_cell WHERE notebook_name = ?1 AND cell_name = ?2), ?4, ?5, ?6)",
    notebook_name: &str,
    cell_name: &str,
    code_notebook_cell_id: &str,
    from_state: &str,
    to_state: &str,
    transition_reason: &str
);

query_sql_rows_no_args!(
    construction_notebook_cells,
    r"  SELECT notebook_name, cell_name, interpretable_code, code_notebook_cell_id
          FROM code_notebook_cell
         WHERE notebook_name = 'ConstructionSqlNotebook'
      ORDER BY cell_name";
    notebook_name: String,
    cell_name: String,
    interpretable_code: String,
    code_notebook_cell_id: String
);

query_sql_rows_no_args!(
    notebook_cells,
    r"SELECT notebook_kernel_id, notebook_name, cell_name, code_notebook_cell_id
        FROM code_notebook_cell";
    notebook_kernel_id: String,
    notebook_name: String,
    cell_name: String,
    code_notebook_cell_id: String
);

execute_sql!(
    upsert_device,
    r"INSERT INTO device (device_id, name, boundary) VALUES (?, ?, ?)
      ON CONFLICT(name, boundary) DO UPDATE SET updated_at = CURRENT_TIMESTAMP
      RETURNING device_id",
    device_id: &str,
    name: &str,
    boundary: &str
);

// // TODO: create infra to be able to validate all SQL in an in-memory SQLite database
// //       by looping through all ExecutableCode blocks and running `prepare`.
// lazy_static! {
//     pub static ref INIT_DDL_EC: ExecutableCode = ExecutableCode::NotebookCell {
//         notebook_name: String::from("ConstructionSqlNotebook"),
//         cell_name: String::from("initialDDL")
//     };
// }

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
    AnonymousSql {
        sql: String,
    },
    Sql {
        identifier: String,
        sql: String,
    },
}

impl<'conn> ExecutableCode {
    pub fn executable_code(&self, ctx: &mut RusqliteContext<'conn>) -> RusqliteResult<String> {
        match self {
            ExecutableCode::NotebookCell {
                notebook_name,
                cell_name,
            } => match select_notebook_cell_code(ctx.conn, notebook_name, cell_name) {
                Ok((_id, code)) => Ok(code),
                Err(err) => Err(err),
            },
            ExecutableCode::AnonymousSql { sql } | ExecutableCode::Sql { sql, .. } => {
                Ok(sql.clone())
            }
        }
    }

    pub fn hash_key(&self) -> String {
        match self {
            ExecutableCode::Sql { identifier, .. } => identifier.clone(),
            ExecutableCode::AnonymousSql { sql } => {
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

pub struct RusqliteContext<'conn> {
    pub conn: &'conn Connection,
    pub bootstrap_result: RusqliteResult<()>,
}

impl<'conn> RusqliteContext<'conn> {
    pub fn new(conn: &'conn rusqlite::Connection) -> RusqliteResult<Self> {
        // TODO: add openDB pragmas https://cj.rs/blog/sqlite-pragma-cheatsheet-for-performance-and-consistency/
        // TODO: use https://www.sqlite.org/pragma.html#pragma_user_version to indicate bootstrap_ddl version/run
        let bootstrap_result = bootstrap_ddl(conn);
        Ok(RusqliteContext {
            conn,
            bootstrap_result,
        })
    }

    pub fn execute_migrations(&mut self) -> RusqliteResult<()> {
        // TODO: should we put these into BEGIN/END TRANSCTION?
        construction_notebook_cells(self.conn, |_index, notebook_name, cell_name, sql, _id| {
            if cell_name.contains("_once_") {
                match self.execute_batch_stateful(
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
                            "[TODO: move this to Otel] {} {} migration not required",
                            notebook_name, cell_name
                        );
                        Ok(())
                    }
                    Some(_) => {
                        println!(
                            "[TODO: move this to Otel] {} {} migrated",
                            notebook_name, cell_name
                        );
                        Ok(())
                    }
                }
            } else {
                println!(
                    "[TODO: move this to Otel] {} {} migrated",
                    notebook_name, cell_name
                );
                self.conn.execute_batch(&sql)
            }
        })
    }

    pub fn prepare(&mut self, ec: &ExecutableCode) -> RusqliteResult<CachedStatement<'conn>> {
        match ec.executable_code(self) {
            Ok(sql) => self.conn.prepare_cached(&sql),
            Err(err) => Err(err),
        }
    }

    pub fn execute_batch(&mut self, ec: &ExecutableCode) -> RusqliteResult<()> {
        match ec.executable_code(self) {
            Ok(sql) => self.conn.execute_batch(&sql),
            Err(err) => Err(err),
        }
    }

    pub fn execute_batch_stateful(
        &mut self,
        ec: &ExecutableCode,
        from_state: &str,
        to_state: &str,
        transition_reason: &str,
    ) -> Option<RusqliteResult<()>> {
        match ec {
            ExecutableCode::NotebookCell {
                notebook_name,
                cell_name,
            } => {
                match is_notebook_cell_state(
                    self.conn,
                    notebook_name,
                    cell_name,
                    from_state,
                    to_state,
                ) {
                    Ok(_) => None,
                    Err(rusqlite::Error::QueryReturnedNoRows) => match self.execute_batch(ec) {
                        Ok(_) => {
                            let ulid = Ulid::new().to_string();
                            match insert_notebook_cell_state(
                                self.conn,
                                notebook_name,
                                cell_name,
                                &ulid,
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
                }
            }
            // TODO: instead of always executing, insert the SQL to code_notebook_cell
            //       in a notebook called `execute_batch_stateful` with ec.hash_key as
            //       the cell name, then recursively call execute_batch_stateful with
            //       that new cell; this way we can track it and only run once
            _ => Some(self.execute_batch(ec)),
        }
    }

    pub fn close(&mut self) -> Result<(), rusqlite::Error> {
        // TODO: add closeDB pragmas
        //       https://cj.rs/blog/sqlite-pragma-cheatsheet-for-performance-and-consistency/
        todo!()
    }

    pub fn upserted_device(&mut self, device: &Device) -> RusqliteResult<usize> {
        let ulid = Ulid::new().to_string();
        upsert_device(
            self.conn,
            &ulid,
            &device.name,
            if let Some(boundary) = &device.boundary {
                boundary
            } else {
                "UNKNOWN"
            },
        )
    }
}
