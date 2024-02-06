use std::fmt::Display;

use derive_new::new;
use pgwire::api::Type;
use sqlparser::ast::{ColumnDef, DataType, Statement};

use crate::error::UdiPgpError;

/// Contains metadata about a specific column in a SQL query, including its name, type, and optional alias.
#[derive(Debug, Clone, new, PartialEq)]
pub struct ColumnMetadata {
    /// Name of the column.
    pub name: String,
    /// Type of expression the column represents (e.g., Standard, Function, Binary).
    pub expr_type: ExpressionType,
    /// Optional alias assigned to the column in the query.
    pub alias: Option<String>,
    /// Type of column
    pub r#type: Type,
}

impl ColumnMetadata {
    pub fn query_session_column() -> Self {
        ColumnMetadata::new(
            "udi_pgp_session_query_id".to_string(),
            ExpressionType::Standard,
            None,
            Type::VARCHAR,
        )
    }
}

impl Default for ColumnMetadata {
    fn default() -> Self {
        ColumnMetadata {
            name: String::new(),
            expr_type: ExpressionType::Standard,
            alias: None,
            r#type: Type::VARCHAR,
        }
    }
}

impl Display for ColumnMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "Column: {} of {} expression and alias: {:?}",
            self.name, self.expr_type, self.alias
        ))
    }
}

/// Enum representing the types of expressions a column in a SQL query can have.
/// Corresponding directly to the types in the statement
#[derive(Debug, Clone, PartialEq)]
pub enum ExpressionType {
    /// A standard column or field.
    Standard,
    /// A column resulting from a binary operation.
    Binary,
    /// A column derived from a function.
    Function,
    /// A compound expression, potentially involving multiple operations or functions.
    Compound,
    /// A wildcard expression, representing multiple or all columns.
    Wildcard,
}

impl Display for ExpressionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExpressionType::Binary => f.write_str("binary"),
            ExpressionType::Function => f.write_str("binary"),
            ExpressionType::Compound => f.write_str("compound"),
            ExpressionType::Standard => f.write_str("standard"),
            ExpressionType::Wildcard => f.write_str("wildcard"),
        }
    }
}

/// Enum representing types of query UDI-PGP will receive
#[derive(Debug, Clone, PartialEq)]
pub enum StmtType {
    /// Driver queries like `SEELCT 1`
    Driver,
    /// A config query sets or updates the configuration. e.g `SET udi_pgp_serve_ncl_core` or `SET udi_pgp_serve_ncl_supplier`
    Config,
    /// Queries to get suppliers and details about each supplier. e.g `SELECT * from udi_pgp_suppier`
    Introspection,
    /// Standard queries to suppliers
    Supplier,
}

/// Represents the metadata of a parsed SQL query, including details about the tables and columns involved.
#[derive(Debug, Clone, PartialEq)]
pub struct UdiPgpStatment {
    /// Names of the tables involved in the query.
    pub tables: Vec<String>,
    /// Metadata about the columns involved in the query.
    pub columns: Vec<ColumnMetadata>,
    pub query: String,
    pub stmt: Statement,
    pub stmt_type: StmtType,
}

impl TryFrom<ColumnDef> for ColumnMetadata {
    type Error = UdiPgpError;

    fn try_from(value: ColumnDef) -> Result<Self, Self::Error> {
        let name = value.name.value;
        let data_type = match value.data_type {
            DataType::BigInt(_) => Type::INT8,
            DataType::Boolean => Type::BOOL,
            DataType::Text => Type::VARCHAR,
            DataType::Binary(_) => Type::BYTEA,
            DataType::Integer(_) => Type::INT4,
            DataType::Double => Type::INT8,
            _ => {
                return Err(UdiPgpError::TypeConversionError(
                    value.data_type.to_string(),
                    "".to_string(),
                ));
            }
        };
        let col = ColumnMetadata {
            name,
            expr_type: ExpressionType::Standard,
            alias: None,
            r#type: data_type,
        };
        Ok(col)
    }
}

impl TryFrom<&ColumnDef> for ColumnMetadata {
    type Error = UdiPgpError;

    fn try_from(value: &ColumnDef) -> Result<Self, Self::Error> {
        let name = &value.name.value;
        let data_type = match value.data_type {
            DataType::BigInt(_) => Type::INT8,
            DataType::Boolean => Type::BOOL,
            DataType::Text => Type::VARCHAR,
            DataType::Binary(_) => Type::BYTEA,
            DataType::Integer(_) => Type::INT4,
            DataType::Double => Type::INT8,
            _ => {
                return Err(UdiPgpError::TypeConversionError(
                    value.data_type.to_string(),
                    "".to_string(),
                ));
            }
        };
        let col = ColumnMetadata {
            name: name.to_string(),
            expr_type: ExpressionType::Standard,
            alias: None,
            r#type: data_type,
        };
        Ok(col)
    }
}
