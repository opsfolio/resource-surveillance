use std::process::Command;

use async_trait::async_trait;
use schema::OsquerySchema;
use serde_json::Value;
use tracing::{debug, error, info};
use udi_pgp::{
    error::{UdiPgpError, UdiPgpResult},
    parser::stmt::{ColumnMetadata, ExpressionType, UdiPgpStatment},
    sql_supplier::SqlSupplier,
    FieldFormat, FieldInfo, Row, UdiPgpModes,
};

mod schema;

#[derive(Debug, Clone)]
pub struct OsquerySupplier {
    pub mode: UdiPgpModes,
}

impl OsquerySupplier {
    pub fn new(mode: UdiPgpModes) -> Self {
        OsquerySupplier { mode }
    }

    //This handles columns/alias that are not actually present in osquery
    //For example, binary operations with alias.
    //e.g  (1<<8) as promisc_flag. The "promisc_flag" is not present
    //in the "interface_details" in osquery
    fn non_standard_column(&self, col: &ColumnMetadata) -> UdiPgpResult<OsquerySchema> {
        // println!("{:#?}", expr);
        match col.expr_type {
            ExpressionType::Binary => Ok(OsquerySchema::new(
                "200".to_string(),
                "".to_string(),
                col.name.clone(),
                "INT".to_string(),
            )),
            _ => {
                error!("Invalid column name: {}", col.name);
                Err(UdiPgpError::SchemaError(
                    format!("Invalid column name: {}", col.name),
                    "".to_string(),
                ))
            }
        }
    }

    fn execute_local_query(
        &self,
        query: &str,
        atc_config_file: Option<String>,
    ) -> UdiPgpResult<Vec<Value>> {
        let mut cmd = Command::new("osqueryi");
        if let Some(cfg_file) = atc_config_file {
            cmd.arg("--config_path").arg(cfg_file);
        }
        cmd.arg("--json").arg(query);
        debug!(
            "Executing osquery with the following args: {:?}",
            cmd.get_args()
        );

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(UdiPgpError::QueryExecutionError("Query failed".to_string()));
        }

        let output_str = String::from_utf8(output.stdout)
            .map_err(|err| UdiPgpError::QueryExecutionError(err.to_string()))?;
        info!("Osquery query executed successfully.");

        let value: Value = serde_json::from_str(&output_str)?;
        value
            .as_array()
            .ok_or(UdiPgpError::QueryExecutionError(
                "Failed to convert json string to array".to_string(),
            ))
            .cloned()
    }

    fn rows(&self, values: &[Value], columns: &[ColumnMetadata]) -> UdiPgpResult<Vec<Vec<Row>>> {
        let mut rows = Vec::with_capacity(values.len());
        for row_value in values {
            let row_object = row_value
                .as_object()
                .ok_or(UdiPgpError::QueryExecutionError(
                    "Row is not an object".to_string(),
                ))?;

            let mut cell_row = Vec::with_capacity(columns.len());
            for col in columns {
                let column_name = col.alias.as_ref().unwrap_or(&col.name);
                let cell = match column_name.as_str() {
                    "ssh_target" | "config_path" => {
                        // let target = ssh_target.as_ref().ok_or("SSH target not found")?;
                        // match self.name() {
                        //     SupplierName::OsqueryAtcLocal => target
                        //         .config_file()
                        //         .ok_or("Config file not found")?
                        //         .to_string(),
                        //     _ => target.ssh_target().to_string(),
                        // }
                        Row::from("".to_string())
                    }
                    "host_id" | "atc_id" => {
                        // let target = ssh_target.as_ref().ok_or("SSH target not found")?;
                        // target.id().to_string()
                        Row::from("".to_string())
                    }
                    _ => {
                        let default = Value::String("".to_string());
                        let val = row_object
                            .get(column_name)
                            .unwrap_or(&default)
                            .as_str()
                            .ok_or(UdiPgpError::QueryExecutionError(
                                "Invalid cell value".to_string(),
                            ))?;
                        Row::from(val.to_string())
                    }
                };
                cell_row.push(cell);
            }
            rows.push(cell_row);
        }

        Ok(rows)
    }
}

#[async_trait]
impl SqlSupplier for OsquerySupplier {
    fn name(&self) -> &str {
        "osquery"
    }

    async fn schema(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<FieldInfo>> {
        let schema = schema::get_schema(&stmt.tables, None)?;
        let columns = &stmt.columns;

        columns
            .iter()
            .map(|col| {
                let col_schema = schema
                    .get(&col.name.to_lowercase())
                    .map(|sch| {
                        let mut sch = sch.clone(); // Clone only once here
                        if let Some(alias) = &col.alias {
                            sch.name = alias.clone();
                        }
                        sch
                    })
                    .unwrap_or_else(|| self.non_standard_column(col).unwrap());

                // impl TryFrom<OsquerySchema> for FieldInfo
                Ok(FieldInfo::new(
                    col_schema.name,
                    None,
                    Some(col_schema.cid.parse::<i16>().unwrap()), // Consider handling the potential parse error
                    col.r#type.clone(),
                    FieldFormat::Text,
                ))
            })
            .collect()
    }

    async fn execute(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<Vec<Row>>> {
        let rows = self.execute_local_query(&stmt.query, None)?;
        self.rows(&rows, &stmt.columns)
    }
}
