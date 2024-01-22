use async_trait::async_trait;
use schema::OsquerySchema;
use tracing::error;
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
        Ok(vec![])
    }
}
