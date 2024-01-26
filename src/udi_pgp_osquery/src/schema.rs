use std::{
    collections::HashMap,
    io::Write,
    process::{Command, Stdio},
    rc::Rc,
};

use anyhow::Context;
use serde::{Deserialize, Serialize};
use tracing::debug;
use udi_pgp::{
    error::{UdiPgpError, UdiPgpResult},
    parser::UdiPgpQueryParser,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OsquerySchema {
    pub cid: String,
    dflt_value: String,
    pub name: String,
    pub notnull: String,
    pk: String,
    // TODO convert this to Type directly. Implement a serde method
    #[serde(rename = "type")]
    pub column_type: String,
    pub table_name: Option<String>,
}

impl OsquerySchema {
    pub fn new(
        cid: String,
        dflt_value: String,
        name: String,
        column_type: String,
    ) -> OsquerySchema {
        OsquerySchema {
            cid,
            dflt_value,
            name,
            notnull: "0".into(),
            pk: "0".into(),
            column_type,
            table_name: None,
        }
    }
}

fn format_schema_query(query: &str) -> String {
    query
        .split(';')
        .next()
        .map(|segment| segment.replace('`', "").replace("HIDDEN", ""))
        .unwrap_or_else(|| "".to_string())
}

pub fn get_schema(
    tables: &Vec<String>,
    atc_config_file: &Option<String>,
) -> UdiPgpResult<HashMap<String, OsquerySchema>> {
    let mut schema_data = Vec::new();

    for table in tables {
        debug!("====== Retrieving schema for {} table ======", table);

        let mut command = Command::new("osqueryi");
        command.arg("--json");

        if let Some(ref cfg_path) = atc_config_file {
            command.arg("--config_path").arg(cfg_path);
        }

        let mut child_process = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(UdiPgpError::IoError)?;

        let stdin = child_process
            .stdin
            .as_mut()
            .context("Failed to open stdin")
            .map_err(|err| UdiPgpError::SchemaError(table.to_string(), err.to_string()))?;

        let query = format!(".schema {}", table);
        stdin.write_all(query.as_bytes())?;

        let output = child_process.wait_with_output()?;

        if !output.status.success() {
            return Err(UdiPgpError::SchemaError(
                table.to_string(),
                "Failed to generate schema. Output Error".to_string(),
            ));
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|err| UdiPgpError::SchemaError(table.to_string(), err.to_string()))?;
        let query = format_schema_query(&output_str);
        if query.is_empty() {
            return Err(UdiPgpError::QueryExecutionError("Schema query definition failed. Please check your query. Expected `CREATE ...`, got empty string".to_string()));
        }
        let stmt = UdiPgpQueryParser::parse(&query, true)?;
        schema_data.push((table.clone(), stmt.columns));
    }

    let mut schemas = HashMap::new();
    for (table_name, columns) in schema_data {
        let table_name_rc = Rc::new(table_name);

        for (idx, col) in columns.into_iter().enumerate() {
            let mut schema = OsquerySchema::new(
                idx.to_string(),
                "".to_string(),
                col.name.to_string(),
                col.r#type.to_string(),
            );
            schema.table_name = Some(Rc::clone(&table_name_rc).to_string());

            schemas.insert(col.name, schema);
        }
    }
    Ok(schemas)
}
