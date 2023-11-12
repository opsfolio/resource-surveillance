/**
 * These are helper macros for creating type-safe functions that wrap SQLite SQL
 * statements in Rusqlite accessors. When you need convenience, use these helpers
 * but if you need high performance (like in a loop or batch), it's probably best
 * to handle them in your own code.
 */

// Macro for executing a non-query SQL command (like INSERT, UPDATE, DELETE) with type-safe bind parameters
macro_rules! execute_sql {
    ($func_name:ident, $sql:expr, $($param_name:ident : $param_type:ty),*) => {
        pub fn $func_name(conn: &Connection $(, $param_name: $param_type)*) -> RusqliteResult<usize> {
            let mut stmt = conn.prepare_cached($sql)?;
            let params = [$(&$param_name as &dyn ToSql),*];
            let affected_rows = stmt.execute(params)?;
            Ok(affected_rows)
        }
    };
}

// Macro for executing a non-query SQL command (like INSERT, UPDATE, DELETE) without bind parameters
macro_rules! _execute_sql_no_args {
    ($func_name:ident, $sql:expr) => {
        pub fn $func_name(conn: &Connection) -> RusqliteResult<usize> {
            let mut stmt = conn.prepare_cached($sql)?;
            let affected_rows = stmt.execute([])?;
            Ok(affected_rows)
        }
    };
}

// Macro for executing multiple queries in a batch without any arguments
macro_rules! execute_sql_batch {
    ($func_name:ident, $sql:expr) => {
        pub fn $func_name(conn: &Connection) -> RusqliteResult<()> {
            conn.execute_batch($sql)?;
            Ok(())
        }
    };
}

// Macro for executing a query SQL command that returns a single row with type-safe bind parameters
macro_rules! query_sql_single {
    // Match when there's only one output column
    ($func_name:ident, $sql:expr, $($param_name:ident : $param_type:ty),*; $out_name:ident : $out_type:ty) => {
        pub fn $func_name(conn: &Connection $(, $param_name: $param_type)*) -> RusqliteResult<$out_type> {
            let mut stmt = conn.prepare_cached($sql)?;
            let params = [$(&$param_name as &dyn ToSql),*];
            let mut rows = stmt.query(params)?;

            if let Some(row) = rows.next()? {
                Ok(row.get::<_, $out_type>(0)?)
            } else {
                Err(rusqlite::Error::QueryReturnedNoRows)
            }
        }
    };
    // Match when there are multiple output columns
    ($func_name:ident, $sql:expr, $($param_name:ident : $param_type:ty),*; $($out_name:ident : $out_type:ty),+) => {
        pub fn $func_name(conn: &Connection $(, $param_name: $param_type)*) -> RusqliteResult<($($out_type),+)> {
            let mut stmt = conn.prepare_cached($sql)?;
            let params = [$(&$param_name as &dyn ToSql),*];
            let mut rows = stmt.query(params)?;

            if let Some(row) = rows.next()? {
                Ok(($(
                    row.get::<_, $out_type>(stringify!($out_name))?,
                )+))
            } else {
                Err(rusqlite::Error::QueryReturnedNoRows)
            }
        }
    };
}

// Macro for executing a query SQL command that returns a single row without bind parameters
macro_rules! _query_sql_single_no_args {
    // Match when there's only one output column
    ($func_name:ident, $sql:expr; $out_name:ident : $out_type:ty) => {
        pub fn $func_name(conn: &Connection) -> RusqliteResult<$out_type> {
            let mut stmt = conn.prepare_cached($sql)?;
            let mut rows = stmt.query([])?;

            if let Some(row) = rows.next()? {
                Ok(row.get::<_, $out_type>(0)?)
            } else {
                Err(rusqlite::Error::QueryReturnedNoRows)
            }
        }
    };
    // Match when there are multiple output columns
    ($func_name:ident, $sql:expr; $($out_name:ident : $out_type:ty),+) => {
        pub fn $func_name(conn: &Connection) -> RusqliteResult<($($out_type),+)> {
            let mut stmt = conn.prepare_cached($sql)?;
            let mut rows = stmt.query([])?;

            if let Some(row) = rows.next()? {
                Ok(($(
                    row.get::<_, $out_type>(stringify!($out_name))?,
                )+))
            } else {
                Err(rusqlite::Error::QueryReturnedNoRows)
            }
        }
    };
}

// Macro for executing a query SQL command that calls a closure for each row with type-safe bind parameters
macro_rules! query_sql_rows {
    // Match when there's only one output column
    ($func_name:ident, $sql:expr, $($param_name:ident : $param_type:ty),*; $out_name:ident : $out_type:ty) => {
        pub fn $func_name<F>(conn: &Connection, mut callback: F $(, $param_name: $param_type)*) -> RusqliteResult<()>
        where
            F: FnMut(usize, $out_type) -> RusqliteResult<()>,
        {
            let mut stmt = conn.prepare_cached($sql)?;
            let params = [$(&$param_name as &dyn ToSql),*];
            let mut rows = stmt.query(params)?;
            let mut row_index = 0;
            while let Some(row) = rows.next()? {
                let value = row.get::<_, $out_type>(0)?;
                callback(row_index, value)?;
                row_index += 1;
            }
            Ok(())
        }
    };
    // Match when there are multiple output columns
    ($func_name:ident, $sql:expr, $($param_name:ident : $param_type:ty),*; $($out_name:ident : $out_type:ty),+) => {
        pub fn $func_name<F>(conn: &Connection, mut callback: F $(, $param_name: $param_type)*) -> RusqliteResult<()>
        where
            F: FnMut(usize, $($out_type),*) -> RusqliteResult<()>,
        {
            let mut stmt = conn.prepare_cached($sql)?;
            let params = [$(&$param_name as &dyn ToSql),*];
            let mut rows = stmt.query(params)?;
            let mut row_index = 0;
            while let Some(row) = rows.next()? {
                callback(row_index, $(row.get::<_, $out_type>(stringify!($out_name))?),*)?;
                row_index += 1;
            }
            Ok(())
        }
    };
}

// Macro for executing a query SQL command that calls a closure for each row without bind parameters
macro_rules! query_sql_rows_no_args {
    // Match when there's only one output column
    ($func_name:ident, $sql:expr; $out_name:ident : $out_type:ty) => {
        pub fn $func_name<F>(conn: &Connection, mut callback: F) -> RusqliteResult<()>
        where
            F: FnMut(usize, $out_type) -> RusqliteResult<()>,
        {
            let mut stmt = conn.prepare_cached($sql)?;
            let mut rows = stmt.query([])?;
            let mut row_index = 0;
            while let Some(row) = rows.next()? {
                let value = row.get::<_, $out_type>(0)?;
                callback(row_index, value)?;
                row_index += 1;
            }
            Ok(())
        }
    };
    // Match when there are multiple output columns
    ($func_name:ident, $sql:expr; $($out_name:ident : $out_type:ty),+) => {
        pub fn $func_name<F>(conn: &Connection, mut callback: F) -> RusqliteResult<()>
        where
            F: FnMut(usize, $($out_type),*) -> RusqliteResult<()>,
        {
            let mut stmt = conn.prepare_cached($sql)?;
            let mut rows = stmt.query([])?;
            let mut row_index = 0;
            while let Some(row) = rows.next()? {
                callback(row_index, $(row.get::<_, $out_type>(stringify!($out_name))?),*)?;
                row_index += 1;
            }
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! query_sql_rows_json {
    ($func_name:ident, $sql:expr, $($param_name:ident : $param_type:ty),*) => {
        pub fn $func_name(conn: &rusqlite::Connection $(, $param_name: $param_type)*) -> rusqlite::Result<serde_json::Value> {
            let mut stmt = conn.prepare_cached($sql)?;
            let params = [$(&$param_name as &dyn rusqlite::ToSql),*];
            let mut rows = stmt.query(params)?;

            let mut array = Vec::new();
            while let Some(row_result) = rows.next()? {
                let mut row_map = serde_json::Map::new();
                for (i, column_name) in row_result.as_ref().column_names().iter().enumerate() {
                    let value: rusqlite::types::ValueRef = row_result.get_ref_unwrap(i);
                    let json_value = match value {
                        rusqlite::types::ValueRef::Null => serde_json::Value::Null,
                        rusqlite::types::ValueRef::Integer(i) => serde_json::Value::from(i),
                        rusqlite::types::ValueRef::Real(f) => serde_json::Value::from(f),
                        rusqlite::types::ValueRef::Text(t) => serde_json::Value::from(String::from_utf8_lossy(t).to_string()),
                        rusqlite::types::ValueRef::Blob(b) => serde_json::Value::from(base64::engine::general_purpose::STANDARD_NO_PAD.encode(b)),
                    };
                    row_map.insert(column_name.to_string(), json_value);
                }
                array.push(serde_json::Value::Object(row_map));
            }

            Ok(serde_json::Value::Array(array))
        }
    };
}
