use std::{collections::HashMap, process::Command, str::FromStr};

use async_trait::async_trait;
use futures::{stream, StreamExt};
use schema::OsquerySchema;
use serde_json::Value;
use tracing::{debug, error, info};
use udi_pgp::{
    config::{Supplier, SupplierType},
    error::{UdiPgpError, UdiPgpResult},
    parser::stmt::{ColumnMetadata, ExpressionType, UdiPgpStatment},
    sql_supplier::{SqlSupplier, SqlSupplierType},
    ssh::{key::SshKey, session::SshTunnelAccess, SshConnection, UdiPgpSshTarget},
    FieldFormat, FieldInfo, Row, Type, UdiPgpModes, FACTORY,
};
use uuid::Uuid;

mod schema;

pub async fn initialize() {
    let mut factory = FACTORY().lock().await;
    factory.register("osquery", generate_new);
}

fn generate_new(supplier: Supplier) -> UdiPgpResult<SqlSupplierType> {
    let sql_suppler = OsquerySupplier {
        mode: supplier.mode,
        atc_file_path: supplier.atc_file_path,
        ssh_targets: supplier.ssh_targets,
        query_session_id: None,
    };
    Ok(Box::new(sql_suppler) as SqlSupplierType)
}

#[derive(Debug, Clone)]
pub struct OsquerySupplier {
    pub mode: UdiPgpModes,
    atc_file_path: Option<String>,
    ssh_targets: Option<Vec<UdiPgpSshTarget>>,
    query_session_id: Option<Uuid>,
}

impl From<Supplier> for OsquerySupplier {
    fn from(value: Supplier) -> Self {
        OsquerySupplier {
            mode: value.mode,
            atc_file_path: value.atc_file_path,
            ssh_targets: value.ssh_targets,
            query_session_id: None,
        }
    }
}

impl From<&Supplier> for OsquerySupplier {
    fn from(value: &Supplier) -> Self {
        OsquerySupplier {
            mode: value.mode.clone(),
            atc_file_path: value.atc_file_path.clone(),
            ssh_targets: value.ssh_targets.clone(),
            query_session_id: None,
        }
    }
}

impl OsquerySupplier {
    pub fn new(mode: UdiPgpModes) -> Self {
        OsquerySupplier {
            mode,
            atc_file_path: None,
            ssh_targets: None,
            query_session_id: None,
        }
    }

    pub fn with_atc_file(&mut self, file: &Option<String>) -> Self {
        self.atc_file_path = file.clone();
        self.clone()
    }

    // TODO handle error
    pub fn with_ssh_targets(&mut self, targets: Vec<String>) -> Self {
        self.ssh_targets = Some(
            targets
                .iter()
                .map(|t| UdiPgpSshTarget::from_str(t.as_str()).unwrap())
                .collect(),
        );
        self.clone()
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

    fn process_columns(
        &self,
        stmt: &mut UdiPgpStatment,
        schema: &mut HashMap<String, OsquerySchema>,
    ) {
        if stmt.columns.len() == 1 && stmt.columns.first().map_or(false, |c| c.name == "*") {
            stmt.columns = schema
                .iter()
                .map(|(_, schema)| ColumnMetadata {
                    name: schema.name.clone(),
                    expr_type: ExpressionType::Standard,
                    alias: None,
                    r#type: Type::VARCHAR,
                })
                .collect();
        } else {
            stmt.columns
                .iter_mut()
                .for_each(|col| col.name = col.name.to_lowercase());
        }
    }

    fn add_remote_specific_columns(&self, stmt: &mut UdiPgpStatment, mode: &UdiPgpModes) {
        if let UdiPgpModes::Remote = mode {
            let remote_columns = ["udi_pgp_ssh_target", "udi_pgp_ssh_host_id"];
            for &name in &remote_columns {
                stmt.columns.push(ColumnMetadata {
                    name: name.to_string(),
                    expr_type: ExpressionType::Standard,
                    alias: None,
                    r#type: Type::VARCHAR,
                });
            }
        }
    }

    fn column_to_field_info(
        &self,
        col: &ColumnMetadata,
        schema: &HashMap<String, OsquerySchema>,
    ) -> UdiPgpResult<FieldInfo> {
        let col_schema = match schema.get(&col.name) {
            Some(col_schema) => col_schema.clone(),
            None => {
                if &col.name == "udi_pgp_session_query_id" {
                    OsquerySchema::new(
                        "100".to_string(),
                        "name".to_owned(),
                        "udi_pgp_session_query_id".to_string(),
                        "TEXT".to_string(),
                    )
                } else if &col.name == "udi_pgp_ssh_host_id" {
                    OsquerySchema::new(
                        "101".to_string(),
                        "name".to_owned(),
                        "udi_pgp_ssh_host_id".to_string(),
                        "TEXT".to_string(),
                    )
                } else if &col.name == "udi_pgp_ssh_target" {
                    OsquerySchema::new(
                        "102".to_string(),
                        "name".to_owned(),
                        "udi_pgp_ssh_target".to_string(),
                        "TEXT".to_string(),
                    )
                } else {
                    self.non_standard_column(col)?
                }
            }
        };

        let cid = col_schema
            .cid
            .parse::<i16>()
            .map_err(|e| UdiPgpError::QueryExecutionError(format!("Failed to parse cid: {}", e)))?;

        let name = match &col.alias {
            Some(alias) => alias.to_string(),
            None => col.name.to_string(),
        };

        let field_info =
            FieldInfo::new(name, None, Some(cid), col.r#type.clone(), FieldFormat::Text);
        Ok(field_info)
    }

    fn execute_local_query(&self, query: &str) -> UdiPgpResult<Vec<Value>> {
        let mut cmd = Command::new("osqueryi");
        if let Some(cfg_file) = &self.atc_file_path {
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

    async fn execute_remote_query(
        &self,
        query: &str,
    ) -> UdiPgpResult<(Vec<Value>, Vec<UdiPgpSshTarget>)> {
        let targets = self.ssh_targets.as_ref().unwrap_or(&vec![]).clone();

        let concurrency_limit = 5;

        let futures = targets.into_iter().map(|target| {
            let query = query.to_owned(); // Clone query to move it into the async block
            async move {
                let addr = match target.port {
                    Some(port) => format!("{}:{}", target.host, port),
                    None => format!("{}:{}", target.host, 22),
                };

                let keypair = SshKey::generate_random().map_err(UdiPgpError::from)?;
                let access = SshTunnelAccess {
                    connection_string: format!("{}@{}", target.user, target.host),
                    keypair,
                };
                let (session, _) = access.create_tunnel(&addr).await?;

                let args = vec!["--json", &query];
                let output = session.execute_command("osqueryi", args).await?;

                let value: Value = serde_json::from_str(&output)?;
                let rows = value
                    .as_array()
                    .ok_or(UdiPgpError::QueryExecutionError(
                        "Failed to convert json string to array".to_string(),
                    ))?
                    .clone();

                Ok::<(Vec<Value>, UdiPgpSshTarget), UdiPgpError>((rows, target.clone()))
            }
        });

        let (successful_results, errors): (Vec<_>, Vec<_>) = stream::iter(futures)
            .buffer_unordered(concurrency_limit)
            .collect::<Vec<UdiPgpResult<(Vec<Value>, UdiPgpSshTarget)>>>()
            .await
            .into_iter()
            .partition(Result::is_ok);

        let rows = successful_results
            .iter()
            .flat_map(|result| result.as_ref().unwrap().0.clone())
            .collect::<Vec<_>>();

        // Extract the SshConnectionParameters part
        let connection_params: Vec<UdiPgpSshTarget> = successful_results
            .into_iter()
            .map(|result| result.unwrap().1) // Extract the SshConnectionParameters
            .collect();
        let errors = errors.into_iter().map(Result::unwrap_err);

        // Log all errors
        for error in errors {
            error!("{}", error);
        }

        Ok((rows, connection_params))
    }

    fn rows(
        &self,
        values: &[Value],
        columns: &[ColumnMetadata],
        targets: Option<Vec<UdiPgpSshTarget>>,
    ) -> UdiPgpResult<Vec<Vec<Row>>> {
        let mut rows = Vec::with_capacity(values.len());

        // infinite iterator of None values to use when targets is None or isn't same length as rows(though this should not happen).
        let default_targets = std::iter::repeat(None);
        let target_iter = targets
            .into_iter()
            .flatten()
            .map(Some)
            .chain(default_targets);

        for (row_value, target) in values.iter().zip(target_iter) {
            let row_object = row_value
                .as_object()
                .ok_or(UdiPgpError::QueryExecutionError(
                    "Row is not an object".to_string(),
                ))?;

            let mut cell_row = Vec::with_capacity(columns.len());
            for col in columns {
                let column_name = col.alias.as_ref().unwrap_or(&col.name);
                let cell = match column_name.as_str() {
                    "udi_pgp_ssh_target" | "config_path" => {
                        // let target = ssh_target.as_ref().ok_or("SSH target not found")?;
                        // match self.name() {
                        //     SupplierName::OsqueryAtcLocal => target
                        //         .config_file()
                        //         .ok_or("Config file not found")?
                        //         .to_string(),
                        //     _ => target.ssh_target().to_string(),
                        // }
                        let value = match target {
                            None => "".to_string(),
                            Some(ref t) => SshConnection::Parameters(t.clone()).to_string(),
                        };
                        Row::from(value)
                    }
                    "udi_pgp_ssh_host_id" | "atc_id" => {
                        // let target = ssh_target.as_ref().ok_or("SSH target not found")?;
                        // target.id().to_string()
                        let value = match target {
                            None => "".to_string(),
                            Some(ref t) => t.id.to_string(),
                        };
                        Row::from(value)
                    }
                    "udi_pgp_session_query_id" => {
                        let value = match self.query_session_id {
                            Some(id) => id.to_string(),
                            None => "null".to_string(),
                        };
                        Row::from(value)
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

    fn supplier_type(&self) -> SupplierType {
        SupplierType::Osquery
    }

    fn update(&mut self, supplier: Supplier) -> UdiPgpResult<()> {
        self.mode = supplier.mode;
        self.atc_file_path = supplier.atc_file_path;
        self.ssh_targets = supplier.ssh_targets;
        Ok(())
    }

    fn add_session_id(&mut self, session_id: Uuid) -> UdiPgpResult<()> {
        self.query_session_id = Some(session_id);
        Ok(())
    }

    fn generate_new(&self, supplier: Supplier) -> UdiPgpResult<SqlSupplierType> {
        generate_new(supplier)
    }

    async fn schema(&mut self, stmt: &mut UdiPgpStatment) -> UdiPgpResult<Vec<FieldInfo>> {
        let mut schema = schema::get_schema(&stmt.tables, &self.atc_file_path)?;
        debug!("{:#?}", stmt.columns);

        // Process columns to either expand "*" or lowercase existing columns
        self.process_columns(stmt, &mut schema);
        self.add_remote_specific_columns(stmt, &self.mode);

        // Always add the query session column
        stmt.columns.push(ColumnMetadata {
            name: "udi_pgp_session_query_id".to_string(),
            expr_type: ExpressionType::Standard,
            alias: None,
            r#type: Type::VARCHAR,
        });

        stmt.columns
            .iter()
            .map(|col| self.column_to_field_info(col, &schema))
            .collect()

        // let mut schema = schema::get_schema(&stmt.tables, &self.atc_file_path)?;
        // debug!("{:#?}", stmt.columns);

        // stmt.columns = if stmt.columns.len() == 1 && stmt.columns.first().unwrap().name == "*" {
        //     schema
        //         .values()
        //         .map(|schema| {
        //             ColumnMetadata::new(
        //                 schema.name.clone(),
        //                 ExpressionType::Standard,
        //                 None,
        //                 Type::VARCHAR,
        //             )
        //         })
        //         .collect::<Vec<_>>()
        // } else {
        //     stmt.columns
        //         .iter()
        //         .map(|col| {
        //             let mut col = col.clone();
        //             col.name = col.name.to_lowercase();
        //             col
        //         })
        //         .collect::<Vec<_>>()
        // };

        // // Add query session
        // stmt.columns.push(ColumnMetadata::new(
        //     "udi_pgp_session_query_id".to_string(),
        //     ExpressionType::Standard,
        //     None,
        //     Type::VARCHAR,
        // ));

        // if let UdiPgpModes::Remote = self.mode {
        //     schema.insert(
        //         "udi_pgp_ssh_target".to_string(),
        //         OsquerySchema::new(
        //             schema.len().to_string(),
        //             "".into(),
        //             "udi_pgp_ssh_target".to_string(),
        //             "TEXT".to_string(),
        //         ),
        //     );
        //     schema.insert(
        //         "udi_pgp_ssh_host_id".to_string(),
        //         OsquerySchema::new(
        //             schema.len().to_string(),
        //             "".into(),
        //             "udi_pgp_ssh_host_id".to_string(),
        //             "TEXT".to_string(),
        //         ),
        //     );

        //     stmt.columns.push(ColumnMetadata::new(
        //         "udi_pgp_ssh_target".to_string(),
        //         ExpressionType::Standard,
        //         None,
        //         Type::VARCHAR,
        //     ));
        //     stmt.columns.push(ColumnMetadata::new(
        //         "udi_pgp_ssh_host_id".to_string(),
        //         ExpressionType::Standard,
        //         None,
        //         Type::VARCHAR,
        //     ));
        // };

        // stmt.columns
        //     .iter()
        //     .map(|col| {
        //         let col_schema = match schema.get(&col.name) {
        //             Some(sch) => {
        //                 let mut sch = sch.clone();
        //                 if let Some(alias) = &col.alias {
        //                     sch.name = alias.clone();
        //                 }
        //                 sch
        //             }
        //             None => self.non_standard_column(col)?,
        //         };

        //         let col_type = col.r#type.clone();
        //         let field_format = FieldFormat::Text;
        //         let cid = col_schema.cid.parse::<i16>().map_err(|e| {
        //             UdiPgpError::QueryExecutionError(format!("Failed to parse cid: {}", e))
        //         })?;

        //         Ok(FieldInfo::new(
        //             col_schema.name,
        //             None,
        //             Some(cid),
        //             col_type,
        //             field_format,
        //         ))
        //     })
        //     .collect()
    }

    async fn execute(&mut self, stmt: &UdiPgpStatment) -> UdiPgpResult<Vec<Vec<Row>>> {
        let (rows, targets) = match self.mode {
            UdiPgpModes::Local => (self.execute_local_query(&stmt.query)?, None),
            UdiPgpModes::Remote => {
                let (rows, targets) = self.execute_remote_query(&stmt.query).await?;
                (rows, Some(targets))
            }
        };
        self.rows(&rows, &stmt.columns, targets)
    }
}
